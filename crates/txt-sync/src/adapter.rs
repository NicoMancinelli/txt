use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use txt_core::history::Change;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("IO failure during sync: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization failure: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Conflict detected at revision {local} vs remote {remote}")]
    Conflict { local: u64, remote: u64 },
    #[error("Network or cloud error: {0}")]
    Network(String),
    #[error("Document not found: {0}")]
    DocumentNotFound(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SyncStatus {
    #[default]
    Synced,
    Syncing,
    Offline,
    Conflict,
    Error,
}

impl SyncStatus {
    pub fn display_badge(&self) -> &'static str {
        match self {
            SyncStatus::Synced => "● Synced",
            SyncStatus::Syncing => "↻ Syncing...",
            SyncStatus::Offline => "○ Offline",
            SyncStatus::Conflict => "⚠ Conflict",
            SyncStatus::Error => "✖ Sync Error",
        }
    }
}

/// A serialized atomic delta/patch for a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncPatch {
    pub patch_id: Uuid,
    pub document_id: Uuid,
    pub revision: u64,
    pub author_id: String,
    pub timestamp: DateTime<Utc>,
    pub changes: Vec<Change>,
    pub base_checksum: u64,
}

impl SyncPatch {
    pub fn new(
        document_id: Uuid,
        revision: u64,
        author_id: impl Into<String>,
        changes: Vec<Change>,
    ) -> Self {
        Self {
            patch_id: Uuid::new_v4(),
            document_id,
            revision,
            author_id: author_id.into(),
            timestamp: Utc::now(),
            changes,
            base_checksum: 0,
        }
    }
}

/// Pluggable interface for synchronization adapters (Local, P2P, Cloud).
pub trait SyncAdapter: Send + Sync {
    /// Adapter identifier name (e.g. "LocalFilesystem", "MemoryRelay", "CloudS3").
    fn name(&self) -> &str;

    /// Pushes a new document patch to the adapter store.
    fn push_patch(&self, patch: &SyncPatch) -> Result<SyncStatus, SyncError>;

    /// Pulls all patches for a document created after `since_revision`.
    fn pull_patches(&self, document_id: &Uuid, since_revision: u64) -> Result<Vec<SyncPatch>, SyncError>;

    /// Current operational status of the adapter.
    fn status(&self) -> SyncStatus;
}
