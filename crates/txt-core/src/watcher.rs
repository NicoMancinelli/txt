use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime};
use crossbeam_channel::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

/// Type of filesystem modification event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchEventKind {
    Created,
    Modified,
    Deleted,
}

/// Event emitted when a watched file is modified, created, or deleted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WatchEvent {
    pub path: PathBuf,
    pub kind: WatchEventKind,
}

impl WatchEvent {
    pub fn created(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            kind: WatchEventKind::Created,
        }
    }

    pub fn modified(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            kind: WatchEventKind::Modified,
        }
    }

    pub fn deleted(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            kind: WatchEventKind::Deleted,
        }
    }
}

/// Metadata snapshot for a file used to detect modifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileSnapshot {
    mtime: SystemTime,
    size: u64,
}

struct WatcherState {
    subscribers: HashMap<usize, Sender<WatchEvent>>,
    worker: Option<JoinHandle<()>>,
}

struct FileSystemWatcherInner {
    state: Mutex<WatcherState>,
    next_id: AtomicUsize,
    running: Arc<AtomicBool>,
    watched_paths: Arc<RwLock<HashSet<PathBuf>>>,
    snapshots: Arc<Mutex<HashMap<PathBuf, FileSnapshot>>>,
    poll_interval: Arc<RwLock<Duration>>,
    debounce_duration: Arc<RwLock<Duration>>,
}

impl FileSystemWatcherInner {
    fn add_subscriber_sender(self: &Arc<Self>, sender: Sender<WatchEvent>) -> usize {
        let mut state = self.state.lock();
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        state.subscribers.insert(id, sender);

        if state.subscribers.len() == 1 && state.worker.is_none() {
            self.start_worker_locked(&mut state);
        }

        id
    }

    fn remove_subscriber(&self, id: usize) {
        let handle_to_join = {
            let mut state = self.state.lock();
            let removed = state.subscribers.remove(&id).is_some();
            if removed && state.subscribers.is_empty() {
                self.running.store(false, Ordering::SeqCst);
                state.worker.take()
            } else {
                None
            }
        };

        if let Some(handle) = handle_to_join {
            let _ = handle.join();
        }
    }

    fn start_worker_locked(self: &Arc<Self>, state: &mut WatcherState) {
        self.running.store(true, Ordering::SeqCst);

        let running = Arc::clone(&self.running);
        let watched_paths = Arc::clone(&self.watched_paths);
        let snapshots = Arc::clone(&self.snapshots);
        let poll_interval = Arc::clone(&self.poll_interval);
        let debounce_duration = Arc::clone(&self.debounce_duration);
        let inner_weak = Arc::downgrade(self);

        let handle = thread::Builder::new()
            .name("txt-fs-watcher".into())
            .spawn(move || {
                Self::worker_loop(
                    running,
                    watched_paths,
                    snapshots,
                    poll_interval,
                    debounce_duration,
                    inner_weak,
                );
            })
            .expect("Failed to spawn fs watcher thread");

        state.worker = Some(handle);
    }

    fn stop_worker(&self) {
        let handle = {
            let mut state = self.state.lock();
            self.running.store(false, Ordering::SeqCst);
            state.worker.take()
        };

        if let Some(h) = handle {
            let _ = h.join();
        }
    }

    fn worker_loop(
        running: Arc<AtomicBool>,
        watched_paths: Arc<RwLock<HashSet<PathBuf>>>,
        snapshots: Arc<Mutex<HashMap<PathBuf, FileSnapshot>>>,
        poll_interval: Arc<RwLock<Duration>>,
        debounce_duration: Arc<RwLock<Duration>>,
        inner_weak: std::sync::Weak<Self>,
    ) {
        let mut pending_events: HashMap<PathBuf, (WatchEventKind, Instant)> = HashMap::new();

        while running.load(Ordering::SeqCst) {
            let roots = watched_paths.read().clone();

            // 1. Scan current files under watched roots
            let mut current_files: HashMap<PathBuf, FileSnapshot> = HashMap::new();
            for root in &roots {
                Self::scan_root_into(root, &mut current_files);
            }

            let now = Instant::now();

            // 2. Compare against recorded snapshots under lock
            {
                let mut snap_guard = snapshots.lock();

                // Detect Created and Modified
                for (path, current_snap) in &current_files {
                    match snap_guard.get(path) {
                        None => {
                            pending_events.insert(path.clone(), (WatchEventKind::Created, now));
                        }
                        Some(prev_snap) => {
                            if prev_snap.mtime != current_snap.mtime || prev_snap.size != current_snap.size {
                                if let Some((existing_kind, _)) = pending_events.get(path) {
                                    if *existing_kind == WatchEventKind::Created {
                                        pending_events.insert(path.clone(), (WatchEventKind::Created, now));
                                    } else {
                                        pending_events.insert(path.clone(), (WatchEventKind::Modified, now));
                                    }
                                } else {
                                    pending_events.insert(path.clone(), (WatchEventKind::Modified, now));
                                }
                            }
                        }
                    }
                }

                // Detect Deleted
                for (path, _) in snap_guard.iter() {
                    let is_under_roots = roots.iter().any(|r| path == r || path.starts_with(r));
                    if is_under_roots && !current_files.contains_key(path) {
                        pending_events.insert(path.clone(), (WatchEventKind::Deleted, now));
                    }
                }

                *snap_guard = current_files;
            }

            // 3. Debounce and collect ready events
            let debounce = *debounce_duration.read();
            let mut ready_events = Vec::new();
            let mut remaining_pending = HashMap::new();

            for (path, (kind, timestamp)) in pending_events {
                if now.duration_since(timestamp) >= debounce {
                    ready_events.push(WatchEvent { path, kind });
                } else {
                    remaining_pending.insert(path, (kind, timestamp));
                }
            }
            pending_events = remaining_pending;

            // 4. Dispatch ready events to subscribers
            if !ready_events.is_empty() {
                if let Some(inner) = inner_weak.upgrade() {
                    let senders: Vec<Sender<WatchEvent>> = {
                        let state = inner.state.lock();
                        state.subscribers.values().cloned().collect()
                    };

                    for event in ready_events {
                        for sender in &senders {
                            let _ = sender.try_send(event.clone());
                        }
                    }
                } else {
                    break;
                }
            }

            // 5. Sleep for poll interval in small slices to be responsive to shutdown
            let interval = *poll_interval.read();
            let sleep_start = Instant::now();
            while sleep_start.elapsed() < interval {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                thread::sleep(Duration::from_millis(5));
            }
        }
    }

    fn scan_root_into(root: &Path, target: &mut HashMap<PathBuf, FileSnapshot>) {
        if !root.exists() {
            return;
        }

        if root.is_file() {
            if let Ok(meta) = root.metadata() {
                let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                let size = meta.len();
                let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
                target.insert(canonical, FileSnapshot { mtime, size });
            }
            return;
        }

        if root.is_dir() {
            for entry in walkdir::WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Ok(meta) = entry.metadata() {
                        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        let size = meta.len();
                        let canonical = entry.path().canonicalize().unwrap_or_else(|_| entry.path().to_path_buf());
                        target.insert(canonical, FileSnapshot { mtime, size });
                    }
                }
            }
        }
    }
}

impl Drop for FileSystemWatcherInner {
    fn drop(&mut self) {
        self.stop_worker();
    }
}

/// An RAII subscription handle.
/// When dropped, automatically unsubscribes from the watcher.
/// If subscriber count drops to zero, the background worker thread is stopped and joined.
pub struct WatchSubscription {
    id: usize,
    receiver: Receiver<WatchEvent>,
    watcher: Arc<FileSystemWatcherInner>,
}

impl WatchSubscription {
    /// Returns the unique ID of this subscription.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Access the event receiver channel for this subscriber.
    pub fn receiver(&self) -> &Receiver<WatchEvent> {
        &self.receiver
    }

    /// Try to receive an event without blocking.
    pub fn try_recv(&self) -> Result<WatchEvent, crossbeam_channel::TryRecvError> {
        self.receiver.try_recv()
    }
}

impl Drop for WatchSubscription {
    fn drop(&mut self) {
        self.watcher.remove_subscriber(self.id);
    }
}

/// Dynamic FileSystemWatcher with strict worker lifecycle management.
///
/// Safety Guardrail:
/// - When subscribers > 0, the background polling thread runs.
/// - When subscribers drop to 0, the background thread stops and is cleaned up immediately.
#[derive(Clone)]
pub struct FileSystemWatcher {
    inner: Arc<FileSystemWatcherInner>,
}

impl Default for FileSystemWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemWatcher {
    /// Creates a new FileSystemWatcher with default intervals (100ms poll, 50ms debounce).
    pub fn new() -> Self {
        Self::with_intervals(Duration::from_millis(100), Duration::from_millis(50))
    }

    /// Creates a new FileSystemWatcher with custom poll interval and debounce duration.
    pub fn with_intervals(poll_interval: Duration, debounce_duration: Duration) -> Self {
        Self {
            inner: Arc::new(FileSystemWatcherInner {
                state: Mutex::new(WatcherState {
                    subscribers: HashMap::new(),
                    worker: None,
                }),
                next_id: AtomicUsize::new(1),
                running: Arc::new(AtomicBool::new(false)),
                watched_paths: Arc::new(RwLock::new(HashSet::new())),
                snapshots: Arc::new(Mutex::new(HashMap::new())),
                poll_interval: Arc::new(RwLock::new(poll_interval)),
                debounce_duration: Arc::new(RwLock::new(debounce_duration)),
            }),
        }
    }

    /// Subscribes to filesystem events with RAII lifecycle cleanup.
    /// Spawns the background worker if this is the first active subscriber.
    pub fn subscribe(&self) -> WatchSubscription {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let id = self.inner.add_subscriber_sender(sender);
        WatchSubscription {
            id,
            receiver,
            watcher: Arc::clone(&self.inner),
        }
    }

    /// Low-level subscription returning a subscription ID.
    pub fn add_subscriber_channel(&self, sender: Sender<WatchEvent>) -> usize {
        self.inner.add_subscriber_sender(sender)
    }

    /// Removes a subscriber by ID.
    /// If active subscriber count drops to zero, the background worker thread is cleanly stopped.
    pub fn remove_subscriber(&self, id: usize) {
        self.inner.remove_subscriber(id);
    }

    /// Returns the current number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.inner.state.lock().subscribers.len()
    }

    /// Returns whether the background polling worker thread is actively running.
    pub fn is_running(&self) -> bool {
        self.inner.running.load(Ordering::SeqCst)
    }

    /// Adds a path (file or directory) to the watch set.
    /// Immediately snapshots existing files under `path` to prevent false Created notifications.
    pub fn watch_path(&self, path: impl AsRef<Path>) {
        let p = path.as_ref();
        let canonical = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        self.inner.watched_paths.write().insert(canonical.clone());

        let mut snap_guard = self.inner.snapshots.lock();
        FileSystemWatcherInner::scan_root_into(&canonical, &mut snap_guard);
    }

    /// Removes a path from the watch set.
    pub fn unwatch_path(&self, path: impl AsRef<Path>) {
        let p = path.as_ref();
        let canonical = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        self.inner.watched_paths.write().remove(&canonical);

        let mut snap_guard = self.inner.snapshots.lock();
        snap_guard.retain(|k, _| !k.starts_with(&canonical) && k != &canonical);
    }

    /// Returns a list of currently watched paths.
    pub fn watched_paths(&self) -> Vec<PathBuf> {
        self.inner.watched_paths.read().iter().cloned().collect()
    }

    /// Sets the polling interval.
    pub fn set_poll_interval(&self, interval: Duration) {
        *self.inner.poll_interval.write() = interval;
    }

    /// Returns the current polling interval.
    pub fn poll_interval(&self) -> Duration {
        *self.inner.poll_interval.read()
    }

    /// Sets the debounce duration.
    pub fn set_debounce_duration(&self, duration: Duration) {
        *self.inner.debounce_duration.write() = duration;
    }

    /// Returns the current debounce duration.
    pub fn debounce_duration(&self) -> Duration {
        *self.inner.debounce_duration.read()
    }

    /// Explicitly stops the background worker thread.
    pub fn stop(&self) {
        self.inner.stop_worker();
    }
}
