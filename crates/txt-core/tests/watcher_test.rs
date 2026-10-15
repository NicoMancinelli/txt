use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;
use txt_core::watcher::{FileSystemWatcher, WatchEventKind};

#[test]
fn test_watcher_dynamic_lifecycle_zero_leak_guardrail() {
    let watcher = FileSystemWatcher::with_intervals(
        Duration::from_millis(20),
        Duration::from_millis(10),
    );

    // Initial state: 0 subscribers, worker NOT running
    assert_eq!(watcher.subscriber_count(), 0);
    assert!(!watcher.is_running());

    // 1st subscriber joins -> worker starts
    let sub1 = watcher.subscribe();
    assert_eq!(watcher.subscriber_count(), 1);
    assert!(watcher.is_running());

    // 2nd subscriber joins -> worker continues running
    let sub2 = watcher.subscribe();
    assert_eq!(watcher.subscriber_count(), 2);
    assert!(watcher.is_running());

    // Drop 1st subscriber -> 1 left, worker still running
    drop(sub1);
    assert_eq!(watcher.subscriber_count(), 1);
    assert!(watcher.is_running());

    // Drop 2nd subscriber -> 0 left, worker MUST stop immediately!
    drop(sub2);
    assert_eq!(watcher.subscriber_count(), 0);
    assert!(!watcher.is_running(), "Worker thread must stop when subscriber count drops to 0");

    // Resubscribing should cleanly restart worker
    let sub3 = watcher.subscribe();
    assert_eq!(watcher.subscriber_count(), 1);
    assert!(watcher.is_running());

    drop(sub3);
    assert_eq!(watcher.subscriber_count(), 0);
    assert!(!watcher.is_running());
}

#[test]
fn test_watcher_detects_created_modified_deleted() {
    let temp = tempdir().unwrap();
    let temp_path = temp.path().canonicalize().unwrap();

    let watcher = FileSystemWatcher::with_intervals(
        Duration::from_millis(20),
        Duration::from_millis(10),
    );
    watcher.watch_path(&temp_path);

    let sub = watcher.subscribe();
    let rx = sub.receiver();

    // 1. Create a file
    let test_file = temp_path.join("sample.txt");
    fs::write(&test_file, "initial content").unwrap();

    let event = rx
        .recv_timeout(Duration::from_millis(1000))
        .expect("Expected Created event");
    assert_eq!(event.kind, WatchEventKind::Created);
    assert_eq!(event.path, test_file);

    // 2. Modify the file
    // Wait slightly so mtime or size changes noticeably
    thread::sleep(Duration::from_millis(50));
    fs::write(&test_file, "modified content that is much longer").unwrap();

    let event = rx
        .recv_timeout(Duration::from_millis(1000))
        .expect("Expected Modified event");
    assert_eq!(event.kind, WatchEventKind::Modified);
    assert_eq!(event.path, test_file);

    // 3. Delete the file
    fs::remove_file(&test_file).unwrap();

    let event = rx
        .recv_timeout(Duration::from_millis(1000))
        .expect("Expected Deleted event");
    assert_eq!(event.kind, WatchEventKind::Deleted);
    assert_eq!(event.path, test_file);
}

#[test]
fn test_watcher_does_not_spam_existing_files_on_init() {
    let temp = tempdir().unwrap();
    let temp_path = temp.path().canonicalize().unwrap();

    // Pre-create existing files before watching
    let file1 = temp_path.join("existing1.txt");
    let file2 = temp_path.join("existing2.txt");
    fs::write(&file1, "1").unwrap();
    fs::write(&file2, "2").unwrap();

    let watcher = FileSystemWatcher::with_intervals(
        Duration::from_millis(20),
        Duration::from_millis(10),
    );
    watcher.watch_path(&temp_path);

    let sub = watcher.subscribe();
    let rx = sub.receiver();

    // Existing files should NOT generate Created events
    let timeout_res = rx.recv_timeout(Duration::from_millis(100));
    assert!(
        timeout_res.is_err(),
        "Existing files should not trigger Created notifications on initial watch"
    );

    // Modifying one of the existing files should generate a Modified event
    thread::sleep(Duration::from_millis(50));
    fs::write(&file1, "updated content").unwrap();

    let event = rx
        .recv_timeout(Duration::from_millis(1000))
        .expect("Expected Modified event on existing file");
    assert_eq!(event.kind, WatchEventKind::Modified);
    assert_eq!(event.path, file1);
}

#[test]
fn test_watcher_broadcasts_to_multiple_subscribers() {
    let temp = tempdir().unwrap();
    let temp_path = temp.path().canonicalize().unwrap();

    let watcher = FileSystemWatcher::with_intervals(
        Duration::from_millis(20),
        Duration::from_millis(10),
    );
    watcher.watch_path(&temp_path);

    let sub1 = watcher.subscribe();
    let sub2 = watcher.subscribe();

    let new_file = temp_path.join("multi.txt");
    fs::write(&new_file, "multi content").unwrap();

    let ev1 = sub1.receiver().recv_timeout(Duration::from_millis(1000)).unwrap();
    let ev2 = sub2.receiver().recv_timeout(Duration::from_millis(1000)).unwrap();

    assert_eq!(ev1.kind, WatchEventKind::Created);
    assert_eq!(ev2.kind, WatchEventKind::Created);
    assert_eq!(ev1.path, new_file);
    assert_eq!(ev2.path, new_file);
}
