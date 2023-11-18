use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;

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

    /// Applies a function to a value associated with a key, returning an owned
    /// result.
    ///
    /// This method is for reading a value with a specified key, processing it
    /// with the given function, and returning the result.
    /// The function must return an owned value, not a reference, to comply with
    /// the volatile nature of the map.
    /// Returns `Some(R)` if the key exists, otherwise `None`.
    pub fn read<R>(&self, key: K, f: impl Fn(&V) -> R) -> Option<R> {
        let lock = self.inner.lock();
        match lock.get(&key) {
            None => None,
            Some(value) => Some(f(value))
        }
    }

    /// Mutates a key's value based on a provided function.
    ///
    /// Performs the update and returns `Some(())` if the key exists, otherwise
    /// `None`.
    pub fn update(&self, key: K, f: impl Fn(&mut V)) -> Option<()> {
        let mut lock = self.inner.lock();
        match lock.get_mut(&key) {
            None => None,
            Some(value) => {
                f(value);
                Some(())
            }
        }
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
