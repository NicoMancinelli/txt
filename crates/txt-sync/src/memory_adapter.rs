use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use crate::adapter::{SyncAdapter, SyncError, SyncPatch, SyncStatus};

/// Fast in-memory sync adapter for ephemeral sessions, P2P relay, and unit testing.
#[derive(Clone, Default)]
pub struct MemorySyncAdapter {
    patches: Arc<RwLock<HashMap<Uuid, Vec<SyncPatch>>>>,
    status: Arc<RwLock<SyncStatus>>,
}

impl MemorySyncAdapter {
    pub fn new() -> Self {
        Self {
            patches: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(SyncStatus::Synced)),
        }
    }

    pub fn set_status(&self, status: SyncStatus) {
        *self.status.write() = status;
    }
}

impl SyncAdapter for MemorySyncAdapter {
    fn name(&self) -> &str {
        "MemorySyncAdapter"
    }

    fn push_patch(&self, patch: &SyncPatch) -> Result<SyncStatus, SyncError> {
        let mut map = self.patches.write();
        let list = map.entry(patch.document_id).or_default();

        // Check if revision is sequential
        if let Some(last) = list.last() {
            if patch.revision <= last.revision {
                *self.status.write() = SyncStatus::Conflict;
                return Err(SyncError::Conflict {
                    local: patch.revision,
                    remote: last.revision,
                });
            }
        }

        list.push(patch.clone());
        *self.status.write() = SyncStatus::Synced;
        Ok(SyncStatus::Synced)
    }

    fn pull_patches(&self, document_id: &Uuid, since_revision: u64) -> Result<Vec<SyncPatch>, SyncError> {
        let map = self.patches.read();
        if let Some(list) = map.get(document_id) {
            let filtered: Vec<SyncPatch> = list
                .iter()
                .filter(|p| p.revision > since_revision)
                .cloned()
                .collect();
            Ok(filtered)
        } else {
            Ok(Vec::new())
        }
    }

    fn status(&self) -> SyncStatus {
        *self.status.read()
    }
}
