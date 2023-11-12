use std::collections::HashMap;
use std::hash::Hash;

use parking_lot::{Mutex, MutexGuard};

/// A synchronization primitive for mutually exclusive access to code based on
/// keys. `KeyLock` allows concurrent access to different keys while
/// guaranteeing exclusive access to the code sections locked by the same key.
#[derive(Debug)]
pub struct KeyLock<K>
where
    K: Eq + Hash + Copy
{
    /// A map holding the locks for each key.
    /// Each key in the map has a corresponding Mutex.
    locks: Mutex<HashMap<K, &'static Mutex<()>>>
}

/// A guard that holds a lock for a given key until it goes out of scope.
/// When `KeyLockGuard` is dropped, the underlying `MutexGuard` is automatically
/// released.
#[derive(Debug)]
pub struct KeyLockGuard<'l> {
    /// The `MutexGuard` for the locked key.
    /// It's bound to the lifetime of the guard to ensure it's not released
    /// prematurely.
    guard: MutexGuard<'l, ()>
}

impl<K> KeyLock<K>
where
    K: Eq + Hash + Copy
{
    /// Creates a new `KeyLock`.
    /// Initializes the internal `HashMap` for storing locks.
    pub fn new() -> Self {
        KeyLock {
            locks: Default::default()
        }
    }

    /// Locks the code section associated with the given key and returns a
    /// guard. If the key does not exist, a new `Mutex` is created, boxed,
    /// leaked, and its static reference is stored in the map.
    /// If the key exists, it tries to lock the corresponding Mutex and returns
    /// a guard that can be used to access the protected code section.
    pub fn lock(&self, key: K) -> KeyLockGuard {
        let lock_ref = {
            let mut locks = self.locks.lock();
            *locks.entry(key).or_insert_with(|| {
                let mutex_box = Box::new(Mutex::new(()));
                Box::leak(mutex_box)
            })
        };

        let guard = lock_ref.lock();
        KeyLockGuard { guard }
    }
}

impl<K> Default for KeyLock<K>
where
    K: Eq + Hash + Copy
{
    /// Provides a default instance of `KeyLock` using the `new` method.
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for KeyLockGuard<'_> {
    /// The drop implementation for `KeyLockGuard`.
    /// This is where you would add any additional logic required to clean up
    /// after the guard is no longer needed. Currently, no action is needed
    /// because the `parking_lot::Mutex` does not poison and will release
    /// the lock automatically.
    fn drop(&mut self) {
        // This space intentionally left blank.
    }
}
