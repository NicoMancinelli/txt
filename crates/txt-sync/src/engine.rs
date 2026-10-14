use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use parking_lot::{Mutex, RwLock};
use uuid::Uuid;
use txt_core::history::Change;
use crate::adapter::{SyncAdapter, SyncError, SyncPatch, SyncStatus};
use crate::lock_manager::ConcurrencyLockManager;

/// Central Synchronization Engine managing adapters, locks, and background sync workers.
pub struct SyncEngine {
    lock_manager: ConcurrencyLockManager<Uuid>,
    adapters: Arc<RwLock<Vec<Arc<dyn SyncAdapter>>>>,
    active_clients: Arc<AtomicUsize>,
    heartbeat_running: Arc<AtomicBool>,
    heartbeat_worker: Arc<Mutex<Option<JoinHandle<()>>>>,
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
