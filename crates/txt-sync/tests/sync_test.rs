use std::sync::Arc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use txt_core::history::Change;
use txt_sync::adapter::{SyncAdapter, SyncStatus};
use txt_sync::cloud_adapter::{CloudConfig, CloudSyncAdapter};
use txt_sync::engine::SyncEngine;
use txt_sync::local_adapter::LocalFilesystemAdapter;
use txt_sync::lock_manager::ConcurrencyLockManager;
use txt_sync::memory_adapter::MemorySyncAdapter;

#[test]
fn test_concurrency_lock_manager() {
    let lock_manager = Arc::new(ConcurrencyLockManager::<String>::new());
    let resource_key = "doc-123".to_string();

    let counter = Arc::new(parking_lot::Mutex::new(0));

    let mut handles = Vec::new();
    for _ in 0..10 {
        let lm = Arc::clone(&lock_manager);
        let key = resource_key.clone();
        let ctr = Arc::clone(&counter);

        handles.push(thread::spawn(move || {
            let _guard = lm.acquire_lock(&key);
            let val = *ctr.lock();
            thread::sleep(Duration::from_millis(5));
            *ctr.lock() = val + 1;
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(*counter.lock(), 10);
}

#[test]
fn test_memory_adapter_push_pull() {
    let adapter = MemorySyncAdapter::new();
    let doc_id = Uuid::new_v4();

    let patch = txt_sync::adapter::SyncPatch::new(
        doc_id,
        1,
        "author-1",
        vec![Change::Insert {
            position: 0,
            text: "fn main() {}".to_string(),
        }],
    );

    let status = adapter.push_patch(&patch).unwrap();
    assert_eq!(status, SyncStatus::Synced);

    let patches = adapter.pull_patches(&doc_id, 0).unwrap();
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].author_id, "author-1");
}

#[test]
fn test_local_filesystem_adapter() {
    let temp_dir = std::env::temp_dir().join(format!("txt_test_{}", Uuid::new_v4()));
    let adapter = LocalFilesystemAdapter::new(temp_dir.clone()).unwrap();

    let doc_id = Uuid::new_v4();
    let patch = txt_sync::adapter::SyncPatch::new(
        doc_id,
        1,
        "local_user",
        vec![Change::Insert {
            position: 0,
            text: "local test content".to_string(),
        }],
    );

    adapter.push_patch(&patch).unwrap();
    let pulled = adapter.pull_patches(&doc_id, 0).unwrap();
    assert_eq!(pulled.len(), 1);

    // Clean up
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_cloud_adapter_offline_buffering() {
    let config = CloudConfig::default();
    let adapter = CloudSyncAdapter::new(config);
    adapter.set_online(false);

    let doc_id = Uuid::new_v4();
    let patch = txt_sync::adapter::SyncPatch::new(
        doc_id,
        1,
        "dev",
        vec![Change::Insert {
            position: 0,
            text: "cloud buffer".to_string(),
        }],
    );

    let status = adapter.push_patch(&patch).unwrap();
    assert_eq!(status, SyncStatus::Offline);
    assert_eq!(adapter.queued_patches_count(), 1);

    adapter.set_online(true);
    let flushed = adapter.flush_offline_queue().unwrap();
    assert_eq!(flushed, 1);
    assert_eq!(adapter.queued_patches_count(), 0);
}

#[test]
fn test_heartbeat_worker_lifecycle_cleanup() {
    let engine = SyncEngine::new();
    assert!(!engine.is_heartbeat_active());
    assert_eq!(engine.active_client_count(), 0);

    // Client connects: starts heartbeat
    engine.register_client();
    assert_eq!(engine.active_client_count(), 1);
    assert!(engine.is_heartbeat_active());

    // Second client connects: stays running
    engine.register_client();
    assert_eq!(engine.active_client_count(), 2);
    assert!(engine.is_heartbeat_active());

    // One disconnects: stays running
    engine.unregister_client();
    assert_eq!(engine.active_client_count(), 1);
    assert!(engine.is_heartbeat_active());

    // Last client disconnects: shuts down heartbeat immediately
    engine.unregister_client();
    assert_eq!(engine.active_client_count(), 0);
    assert!(!engine.is_heartbeat_active());
}
