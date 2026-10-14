use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use crate::adapter::{SyncAdapter, SyncError, SyncPatch, SyncStatus};

/// Offline-first local filesystem sync adapter with atomic journal writes.
#[derive(Clone)]
pub struct LocalFilesystemAdapter {
    storage_dir: PathBuf,
    status: Arc<RwLock<SyncStatus>>,
}

impl LocalFilesystemAdapter {
    pub fn new(storage_dir: PathBuf) -> Result<Self, std::io::Error> {
        fs::create_dir_all(&storage_dir)?;
        Ok(Self {
            storage_dir,
            status: Arc::new(RwLock::new(SyncStatus::Synced)),
        })
    }

    fn doc_dir(&self, document_id: &Uuid) -> PathBuf {
        self.storage_dir.join(document_id.to_string())
    }
}

impl SyncAdapter for LocalFilesystemAdapter {
    fn name(&self) -> &str {
        "LocalFilesystemAdapter"
    }

    fn push_patch(&self, patch: &SyncPatch) -> Result<SyncStatus, SyncError> {
        let dir = self.doc_dir(&patch.document_id);
        fs::create_dir_all(&dir)?;

        let patch_filename = format!("{:010}_{}.json", patch.revision, patch.patch_id);
        let final_path = dir.join(&patch_filename);
        let temp_path = dir.join(format!("{}.tmp", patch_filename));

        let data = serde_json::to_vec_pretty(patch)?;

        // Atomic write via temp file + rename
        {
            let mut file = File::create(&temp_path)?;
            file.write_all(&data)?;
            file.sync_all()?;
        }
        fs::rename(temp_path, final_path)?;

        *self.status.write() = SyncStatus::Synced;
        Ok(SyncStatus::Synced)
    }

    fn pull_patches(&self, document_id: &Uuid, since_revision: u64) -> Result<Vec<SyncPatch>, SyncError> {
        let dir = self.doc_dir(document_id);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut patches = Vec::new();
        let entries = fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(patch) = serde_json::from_str::<SyncPatch>(&content) {
                    if patch.revision > since_revision {
                        patches.push(patch);
                    }
                }
            }
        }

        patches.sort_by_key(|p| p.revision);
        Ok(patches)
    }

    fn status(&self) -> SyncStatus {
        *self.status.read()
    }
}
