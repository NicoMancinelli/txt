use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use parking_lot::{Mutex, RwLock};
use uuid::Uuid;
use txt_core::history::Change;
use crate::adapter::{SyncAdapter, SyncError, SyncPatch, SyncStatus};
use crate::crdt::{CrdtBufferState, CrdtOp, CrdtPatch};
use crate::lock_manager::ConcurrencyLockManager;

/// Central Synchronization Engine managing adapters, locks, and background sync workers.
pub struct SyncEngine {
    lock_manager: ConcurrencyLockManager<Uuid>,
    adapters: Arc<RwLock<Vec<Arc<dyn SyncAdapter>>>>,
    active_clients: Arc<AtomicUsize>,
    heartbeat_running: Arc<AtomicBool>,
    heartbeat_worker: Arc<Mutex<Option<JoinHandle<()>>>>,
    crdt_buffers: Arc<RwLock<HashMap<Uuid, CrdtBufferState>>>,
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncEngine {
    pub fn new() -> Self {
        Self {
            lock_manager: ConcurrencyLockManager::new(),
            adapters: Arc::new(RwLock::new(Vec::new())),
            active_clients: Arc::new(AtomicUsize::new(0)),
            heartbeat_running: Arc::new(AtomicBool::new(false)),
            heartbeat_worker: Arc::new(Mutex::new(None)),
            crdt_buffers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds a synchronization adapter to the engine.
    pub fn add_adapter(&self, adapter: Arc<dyn SyncAdapter>) {
        self.adapters.write().push(adapter);
    }

    /// Applies and pushes document changes across all registered adapters.
    /// Serialized per-document via ConcurrencyLockManager to prevent race conditions.
    pub fn sync_document_changes(
        &self,
        document_id: Uuid,
        revision: u64,
        author: &str,
        changes: Vec<Change>,
    ) -> Result<Vec<SyncStatus>, SyncError> {
        // Serialized lock per document resource
        let _guard = self.lock_manager.acquire_lock(&document_id);

        let patch = SyncPatch::new(document_id, revision, author, changes);
        let mut results = Vec::new();

        let adapters = self.adapters.read();
        for adapter in adapters.iter() {
            let status = adapter.push_patch(&patch)?;
            results.push(status);
        }

        Ok(results)
    }

    /// Pulls latest patches for a document across all adapters.
    pub fn pull_document_patches(
        &self,
        document_id: &Uuid,
        since_revision: u64,
    ) -> Result<Vec<SyncPatch>, SyncError> {
        let _guard = self.lock_manager.acquire_lock(document_id);

        let mut all_patches = Vec::new();
        let adapters = self.adapters.read();
        for adapter in adapters.iter() {
            let mut patches = adapter.pull_patches(document_id, since_revision)?;
            all_patches.append(&mut patches);
        }

        all_patches.sort_by_key(|p| p.revision);
        all_patches.dedup_by_key(|p| p.patch_id);
        Ok(all_patches)
    }

    /// Initializes a CRDT buffer for a document under ConcurrencyLockManager.
    pub fn init_crdt_buffer(
        &self,
        document_id: Uuid,
        peer_id: &str,
        initial_text: &str,
    ) {
        let _guard = self.lock_manager.acquire_lock(&document_id);
        let mut buffers = self.crdt_buffers.write();
        buffers.insert(
            document_id,
            CrdtBufferState::from_str(document_id, peer_id, initial_text),
        );
    }

    /// Applies a remote CRDT operation under ConcurrencyLockManager.
    pub fn apply_crdt_op(
        &self,
        document_id: Uuid,
        op: CrdtOp,
    ) -> Result<bool, SyncError> {
        let _guard = self.lock_manager.acquire_lock(&document_id);
        let mut buffers = self.crdt_buffers.write();
        let buffer = buffers
            .entry(document_id)
            .or_insert_with(|| CrdtBufferState::new(document_id, op.peer_id()));
        Ok(buffer.apply_remote_op(op))
    }

    /// Merges an incoming CRDT patch under ConcurrencyLockManager.
    pub fn apply_crdt_patch(&self, patch: &CrdtPatch) -> Result<usize, SyncError> {
        let _guard = self.lock_manager.acquire_lock(&patch.document_id);
        let mut buffers = self.crdt_buffers.write();
        let buffer = buffers
            .entry(patch.document_id)
            .or_insert_with(|| CrdtBufferState::new(patch.document_id, &patch.peer_id));
        Ok(buffer.apply_patch(patch))
    }

    /// Generates and drains an uncommitted CRDT patch under ConcurrencyLockManager.
    pub fn generate_crdt_patch(
        &self,
        document_id: Uuid,
    ) -> Result<Option<CrdtPatch>, SyncError> {
        let _guard = self.lock_manager.acquire_lock(&document_id);
        let mut buffers = self.crdt_buffers.write();
        if let Some(buffer) = buffers.get_mut(&document_id) {
            Ok(Some(buffer.generate_patch()))
        } else {
            Ok(None)
        }
    }

    /// Applies a CRDT patch under ConcurrencyLockManager and pushes to all registered sync adapters.
    pub fn sync_crdt_patch(
        &self,
        patch: &CrdtPatch,
        revision: u64,
    ) -> Result<Vec<SyncStatus>, SyncError> {
        let _guard = self.lock_manager.acquire_lock(&patch.document_id);
        {
            let mut buffers = self.crdt_buffers.write();
            let buffer = buffers
                .entry(patch.document_id)
                .or_insert_with(|| CrdtBufferState::new(patch.document_id, &patch.peer_id));
            buffer.apply_patch(patch);
        }

        let sync_patch = patch.to_sync_patch(revision);
        let mut results = Vec::new();
        let adapters = self.adapters.read();
        for adapter in adapters.iter() {
            let status = adapter.push_patch(&sync_patch)?;
            results.push(status);
        }

        Ok(results)
    }

    /// Performs a local string insertion under ConcurrencyLockManager.
    pub fn local_crdt_insert(
        &self,
        document_id: Uuid,
        peer_id: &str,
        position: usize,
        text: &str,
    ) -> Result<Vec<CrdtOp>, SyncError> {
        let _guard = self.lock_manager.acquire_lock(&document_id);
        let mut buffers = self.crdt_buffers.write();
        let buffer = buffers
            .entry(document_id)
            .or_insert_with(|| CrdtBufferState::new(document_id, peer_id));
        Ok(buffer.local_insert_str(position, text))
    }

    /// Performs a local deletion under ConcurrencyLockManager.
    pub fn local_crdt_delete(
        &self,
        document_id: Uuid,
        peer_id: &str,
        position: usize,
    ) -> Result<Option<CrdtOp>, SyncError> {
        let _guard = self.lock_manager.acquire_lock(&document_id);
        let mut buffers = self.crdt_buffers.write();
        let buffer = buffers
            .entry(document_id)
            .or_insert_with(|| CrdtBufferState::new(document_id, peer_id));
        Ok(buffer.local_delete(position))
    }

    /// Reconstructs full visible text for a CRDT document under ConcurrencyLockManager.
    pub fn get_crdt_text(&self, document_id: &Uuid) -> Option<String> {
        let _guard = self.lock_manager.acquire_lock(document_id);
        let buffers = self.crdt_buffers.read();
        buffers.get(document_id).map(|b| b.to_string())
    }

    /// Returns a clone of the CRDT buffer state under ConcurrencyLockManager.
    pub fn get_crdt_state(&self, document_id: &Uuid) -> Option<CrdtBufferState> {
        let _guard = self.lock_manager.acquire_lock(document_id);
        let buffers = self.crdt_buffers.read();
        buffers.get(document_id).cloned()
    }

    /// Client connected: increment active client counter and start background worker if 0 -> 1.
    pub fn register_client(&self) -> usize {
        let prev = self.active_clients.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            self.start_heartbeat_worker();
        }
        prev + 1
    }

    /// Client disconnected: decrement counter and clean up background worker if reaching 0.
    pub fn unregister_client(&self) -> usize {
        let prev = self.active_clients.fetch_sub(1, Ordering::SeqCst);
        let count = prev.saturating_sub(1);
        if count == 0 {
            self.stop_heartbeat_worker();
        }
        count
    }

    pub fn active_client_count(&self) -> usize {
        self.active_clients.load(Ordering::SeqCst)
    }

    pub fn is_heartbeat_active(&self) -> bool {
        self.heartbeat_running.load(Ordering::SeqCst)
    }

    fn start_heartbeat_worker(&self) {
        let mut worker_lock = self.heartbeat_worker.lock();
        if worker_lock.is_some() {
            return;
        }

        self.heartbeat_running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.heartbeat_running);
        let adapters = Arc::clone(&self.adapters);

        let handle = thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(50));
                // Periodic health probe on registered adapters
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                let list = adapters.read();
                for adapter in list.iter() {
                    let _ = adapter.status();
                }
            }
        });

        *worker_lock = Some(handle);
    }

    fn stop_heartbeat_worker(&self) {
        self.heartbeat_running.store(false, Ordering::SeqCst);
        let mut worker_lock = self.heartbeat_worker.lock();
        if let Some(handle) = worker_lock.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for SyncEngine {
    fn drop(&mut self) {
        self.stop_heartbeat_worker();
    }
}
