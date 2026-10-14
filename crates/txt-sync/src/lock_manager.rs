use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use parking_lot::Mutex;

/// Concurrency Lock Manager providing per-resource serialized locking.
/// Ensures sequential, race-free state updates on shared document resources.
#[derive(Clone)]
pub struct ConcurrencyLockManager<K: Hash + Eq + Clone> {
    locks: Arc<Mutex<HashMap<K, Arc<Mutex<()>>>>>,
}

impl<K: Hash + Eq + Clone> Default for ConcurrencyLockManager<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq + Clone> ConcurrencyLockManager<K> {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Acquires a serialized lock guard for a specific resource key.
    /// Operations on the same resource key will block until the guard is dropped.
    pub fn acquire_lock(&self, key: &K) -> ResourceLockGuard {
        let lock = {
            let mut map = self.locks.lock();
            map.entry(key.clone())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };

        let guard = lock.lock_arc();
        ResourceLockGuard { _guard: guard }
    }

    /// Executes a closure under the resource lock.
    pub fn with_lock<T, F>(&self, key: &K, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _guard = self.acquire_lock(key);
        f()
    }

    /// Prunes unused locks from memory.
    pub fn prune_idle(&self) {
        let mut map = self.locks.lock();
        map.retain(|_, lock| Arc::strong_count(lock) > 1);
    }

    /// Number of active lock entries.
    pub fn active_locks_count(&self) -> usize {
        self.locks.lock().len()
    }
}

pub struct ResourceLockGuard {
    _guard: parking_lot::ArcMutexGuard<parking_lot::RawMutex, ()>,
}
