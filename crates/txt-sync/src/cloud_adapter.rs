use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use crate::adapter::{SyncAdapter, SyncError, SyncPatch, SyncStatus};

/// Cloud sync configuration options.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CloudConfig {
    pub endpoint_url: String,
    pub api_token: Option<String>,
    pub auto_sync_interval_secs: u64,
    pub enable_offline_buffering: bool,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            endpoint_url: "https://api.txt-editor.dev/v1/sync".to_string(),
            api_token: None,
            auto_sync_interval_secs: 30,
            enable_offline_buffering: true,
        }
    }
}

/// Pluggable Cloud Sync Adapter with offline buffer queue.
#[derive(Clone)]
pub struct CloudSyncAdapter {
    config: Arc<RwLock<CloudConfig>>,
    offline_queue: Arc<RwLock<Vec<SyncPatch>>>,
    status: Arc<RwLock<SyncStatus>>,
}

impl CloudSyncAdapter {
    pub fn new(config: CloudConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            offline_queue: Arc::new(RwLock::new(Vec::new())),
            status: Arc::new(RwLock::new(SyncStatus::Offline)),
        }
    }

    pub fn set_online(&self, online: bool) {
        let mut st = self.status.write();
        *st = if online {
            SyncStatus::Synced
        } else {
            SyncStatus::Offline
        };
    }

    pub fn queued_patches_count(&self) -> usize {
        self.offline_queue.read().len()
    }

    pub fn flush_offline_queue(&self) -> Result<usize, SyncError> {
        let mut queue = self.offline_queue.write();
        let count = queue.len();
        // In full network mode, these patches would be sent to endpoint_url
        queue.clear();
        *self.status.write() = SyncStatus::Synced;
        Ok(count)
    }
}

impl SyncAdapter for CloudSyncAdapter {
    fn name(&self) -> &str {
        "CloudSyncAdapter"
    }

    fn push_patch(&self, patch: &SyncPatch) -> Result<SyncStatus, SyncError> {
        let status = *self.status.read();
        if status == SyncStatus::Offline {
            if self.config.read().enable_offline_buffering {
                self.offline_queue.write().push(patch.clone());
                return Ok(SyncStatus::Offline);
            } else {
                return Err(SyncError::Network("Cloud service is offline".to_string()));
            }
        }

        // Simulate successful push to cloud
        *self.status.write() = SyncStatus::Synced;
        Ok(SyncStatus::Synced)
    }

    fn pull_patches(&self, _document_id: &Uuid, _since_revision: u64) -> Result<Vec<SyncPatch>, SyncError> {
        let status = *self.status.read();
        if status == SyncStatus::Offline {
            return Ok(Vec::new());
        }
        // Cloud pull returns newly fetched patches
        Ok(Vec::new())
    }

    fn status(&self) -> SyncStatus {
        *self.status.read()
    }
}
