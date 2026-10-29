use std::collections::HashMap;
use std::hash::Hash;

use parking_lot::{Mutex, MutexGuard};
use tokio::sync::Notify;

/// A synchronization primitive for mutually exclusive access to code based on
/// keys.
///
/// `KeyLock` allows concurrent access to different keys while
/// guaranteeing exclusive access to the code sections locked by the same key.
#[derive(Debug)]
pub struct KeyLock<K>
where
    K: Eq + Hash + Copy
{
    /// A map holding the locks for each key.
    /// Each key in the map has a corresponding [`Notify`].
    locks: Mutex<HashMap<K, &'static Notify>>
}

/// A guard that holds a lock for a given key until it goes out of scope.
///
/// When [`KeyLockGuard`] is dropped, the lock for that key is released.
#[derive(Debug)]
#[must_use]
pub struct KeyLockGuard {
    /// The notify to wake up a waiting task.
    ///
    /// Since we keep the `Notify` until the end of the program, we don't need
    /// reference the lifetime here better.
    notify: &'static Notify
}

impl<K> KeyLock<K>
where
    K: Eq + Hash + Copy
{
    /// Creates a new `KeyLock`.
    /// Initializes the internal `HashMap` for storing locks.
    pub fn new() -> Self {
        KeyLock { locks: Default::default() }
    }

    /// Locks the code section associated with the given key and returns a
    /// guard.
    ///
    /// If the key does not exist, a new [`Notify`] is created, boxed,
    /// leaked, and its static reference is stored in the map.
    ///
    /// Then this waits on the `Notify` to resolve.
    /// Only one task can be waken up by this as only a newly created `Notify`
    /// notifies or a dropped `KeyLockGuard`.
    pub async fn lock(&self, key: K) -> KeyLockGuard {
        let lock_ref = {
            let mut locks = self.locks.lock();
            *locks.entry(key).or_insert_with(|| {
                let mutex_box = Box::new(Notify::new());
                // instantly notify so that the next request will resolve
                // immediately
                mutex_box.notify_one();
                Box::leak(mutex_box)
            })
        };

        // wait here until one Notify notifies
        lock_ref.notified().await;
        KeyLockGuard { notify: lock_ref }
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

impl Drop for KeyLockGuard {
    /// On drop, notify a waiting task.
    ///
    /// The internal [`Notify::notify_one`] will be called here ensuring that
    /// another task may start now.
    fn drop(&mut self) {
        self.notify.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::{Arc, LazyLock};
    use std::time::Duration;

    use dashmap::DashSet;
    use itertools::Itertools;
    use tokio::task::JoinSet;
    use tokio::time::Timeout;
    use tokio::time::error::Elapsed;

    use super::*;

    fn leak<T>(to_leak: T) -> &'static T {
        Box::leak(Box::new(to_leak))
    }

    async fn sleep() {
        tokio::time::sleep(Duration::from_millis(30)).await;
    }

    async fn timeout<F: IntoFuture>(future: F) -> Result<F::Output, Elapsed> {
        tokio::time::timeout(Duration::from_millis(30), future).await
    }

    #[tokio::test]
    async fn single_key_sequence() {
        const N: usize = 10;

        let key_lock = leak(KeyLock::new());
        let barrier = leak(Notify::new());
        let bucket0 = leak(DashSet::with_capacity(N));
        let bucket1 = leak(DashSet::with_capacity(N));
        let bucket2 = leak(DashSet::with_capacity(N));

        let mut tasks = JoinSet::new();
        for idx in 1..=N {
            tasks.spawn(async move {
                bucket0.insert(idx);
                barrier.notified().await;

                let guard = key_lock.lock(0).await;
                bucket1.insert(idx);
                barrier.notified().await;
                drop(guard);

                let guard = key_lock.lock(idx).await;
                bucket2.insert(idx);
                barrier.notified().await;
                drop(guard);
            });
        }

        sleep().await;
        assert_eq!(bucket0.len(), N);
        barrier.notify_waiters();

        for count in 1..=N {
            sleep().await;
            assert_eq!(bucket1.len(), count);
            barrier.notify_waiters();
        }

        sleep().await;
        assert_eq!(bucket2.len(), N);
        barrier.notify_waiters();

        assert!(timeout(tasks.join_all()).await.is_ok());
    }
}
