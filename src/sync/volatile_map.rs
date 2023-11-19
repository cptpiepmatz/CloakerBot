use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;

use futures::FutureExt;
use parking_lot::Mutex;

/// Thread-safe map with automatic entry removal.
///
/// This map uses guards to manage the lifecycle of its entries, removing them
/// when the guards are dropped.
#[derive(Debug)]
pub struct VolatileMap<K, V> {
    inner: Mutex<HashMap<K, V>>
}

/// Guard for [`VolatileMap`] entries, ensuring their temporary existence.
///
/// Created on entry insertion, this guard removes its associated entry upon
/// being dropped.
#[must_use]
#[derive(Debug)]
pub struct VolatileMapGuard<'m, K, V>
where
    K: Eq + Hash + Copy
{
    key: K,
    map_ref: &'m VolatileMap<K, V>
}

impl<K, V> VolatileMap<K, V>
where
    K: Eq + Hash + Copy
{
    /// Creates a new empty `VolatileMap`.
    pub fn new() -> Self {
        Self {
            inner: Default::default()
        }
    }

    /// Inserts a value with a guard if the key is vacant.
    ///
    /// Returns a guard on successful insertion, or `None` if the key is
    /// occupied.
    #[must_use]
    pub fn upsert(&self, key: K, value: V) -> Option<VolatileMapGuard<K, V>> {
        let mut lock = self.inner.lock();
        match lock.entry(key) {
            Entry::Occupied(_) => None,
            Entry::Vacant(entry) => {
                entry.insert(value);
                Some(VolatileMapGuard {
                    key,
                    map_ref: &self
                })
            }
        }
    }

    /// Applies a function to a value associated with a key.
    ///
    /// This method allows the application of a function to the value of a given
    /// key, which can either modify the value or return a computed result.
    /// The function receives a mutable reference to the value (`&mut V`) and
    /// returns a result of type `R`. If the key is found, `Some(R)` is
    /// returned; otherwise, it returns `None`.
    pub fn apply<R>(&self, key: K, f: impl Fn(&mut V) -> R) -> Option<R> {
        let mut lock = self.inner.lock();
        lock.get_mut(&key).map(f)
    }
}

impl<K, V> Default for VolatileMap<K, V>
where
    K: Eq + Hash + Copy
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Drop for VolatileMapGuard<'_, K, V>
where
    K: Eq + Hash + Copy
{
    /// Automatically removes the key-value pair on guard drop.
    ///
    /// Ensures entry removal, maintaining the map's volatile nature.
    fn drop(&mut self) {
        let mut lock = self.map_ref.inner.lock();
        lock.remove(&self.key);
    }
}
