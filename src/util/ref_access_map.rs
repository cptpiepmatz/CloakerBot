use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::mem;

use serde::{Deserialize, Serialize};

/// A data structure for storing values accessible by two keys.
///
/// `RefAccessMap` allows values (`V`) to be accessed using two different keys:
/// a reference key (`R`) and an accessor key (`A`). This structure is
/// particularly useful in scenarios where multiple references are associated
/// with a single key that maps to the actual data. For example, in server-host
/// scenarios, `R` could represent server IDs, `A` could represent host IDs, and
/// `V` could be the data associated with each host.
///
/// # Examples
///
/// ```
/// let mut map = RefAccessMap::new();
/// // Suppose `server1` and `server2` are hosted by `host1`
/// map.insert("server1", "host1", "Host1Data");
/// map.insert_ref("server2", "host1");
///
/// assert_eq!(map.ref_get(&"server1"), Some(&"Host1Data"));
/// assert_eq!(map.acc_get(&"host1"), Some(&"Host1Data"));
/// assert_eq!(map.acc_get(&"server2"), Some(&"Host1Data"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(into = "RefAccessStore<R, A, V>", from = "RefAccessStore<R, A, V>")]
pub struct RefAccessMap<R, A, V>
where
    R: Eq + Hash + Clone,
    A: Eq + Hash + Clone,
    V: Clone
{
    reference_map: HashMap<R, A>,
    accessor_map: HashMap<A, V>
}

impl<R, A, V> RefAccessMap<R, A, V>
where
    R: Eq + Hash + Clone,
    A: Eq + Hash + Clone,
    V: Clone
{
    /// Creates a new, empty `RefAccessMap`.
    pub fn new() -> Self {
        Self {
            reference_map: Default::default(),
            accessor_map: Default::default()
        }
    }

    /// Retrieves a reference to a value by a reference key.
    ///
    /// Returns `None` if the key does not map to any accessor or the accessor
    /// is absent.
    pub fn ref_get(&self, ref_key: &R) -> Option<&V> {
        let acc_key = self.reference_map.get(ref_key)?;
        self.accessor_map.get(acc_key)
    }

    /// Retrieves a reference to a value by an accessor key.
    ///
    /// Returns `None` if the accessor key is not present in the map.
    pub fn acc_get(&self, acc_key: &A) -> Option<&V> {
        self.accessor_map.get(acc_key)
    }

    /// Retrieves a mutable reference to a value by a reference key.
    ///
    /// Returns `None` if the key does not map to any accessor or the accessor
    /// is absent.
    pub fn ref_get_mut(&mut self, ref_key: &R) -> Option<&mut V> {
        let acc_key = self.reference_map.get(ref_key)?;
        self.accessor_map.get_mut(acc_key)
    }

    /// Retrieves a mutable reference to a value by an accessor key.
    ///
    /// Returns `None` if the accessor key is not present in the map.
    pub fn acc_get_mut(&mut self, acc_key: &A) -> Option<&mut V> {
        self.accessor_map.get_mut(acc_key)
    }

    /// Inserts a value with associated reference and accessor keys.
    ///
    /// Returns a tuple with the old accessor key and value if the reference key
    /// was already present, or `(None, None)` otherwise.
    pub fn insert(&mut self, ref_key: R, acc_key: A, value: V) -> (Option<A>, Option<V>) {
        let ref_opt = self.reference_map.insert(ref_key, acc_key.clone());
        let acc_opt = self.accessor_map.insert(acc_key, value);
        (ref_opt, acc_opt)
    }

    /// Inserts a reference-accessor pair, ignoring the value.
    ///
    /// Returns the old accessor if the reference key was already present, or
    /// `None` otherwise.
    pub fn insert_ref(&mut self, ref_key: R, acc_key: A) -> Option<A> {
        self.reference_map.insert(ref_key, acc_key)
    }
}

impl<R, A, V> Default for RefAccessMap<R, A, V> where
    R: Eq + Hash + Clone,
    A: Eq + Hash + Clone,
    V: Clone {
    fn default() -> Self {
        Self {
            reference_map: Default::default(),
            accessor_map: Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefAccessStoreEntry<R, V>
where
    R: Eq + Hash + Clone,
    V: Clone
{
    value: V,
    refs: HashSet<R>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefAccessStore<R, A, V>(HashMap<A, RefAccessStoreEntry<R, V>>)
where
    R: Eq + Hash + Clone,
    A: Eq + Hash + Clone,
    V: Clone;

impl<R, A, V> From<RefAccessMap<R, A, V>> for RefAccessStore<R, A, V>
where
    R: Eq + Hash + Clone,
    A: Eq + Hash + Clone,
    V: Clone
{
    fn from(value: RefAccessMap<R, A, V>) -> Self {
        let RefAccessMap {
            reference_map,
            accessor_map
        } = value;

        let mut entries: HashMap<A, RefAccessStoreEntry<R, V>> = accessor_map
            .into_iter()
            .map(|(k, v)| {
                (k, RefAccessStoreEntry {
                    value: v,
                    refs: HashSet::new()
                })
            })
            .collect();

        for (ref_k, ref_v) in reference_map.into_iter() {
            if let Some(entry) = entries.get_mut(&ref_v) {
                entry.refs.insert(ref_k);
            }
        }

        Self(entries)
    }
}

impl<R, A, V> From<RefAccessStore<R, A, V>> for RefAccessMap<R, A, V>
where
    R: Eq + Hash + Clone,
    A: Eq + Hash + Clone,
    V: Clone
{
    fn from(mut value: RefAccessStore<R, A, V>) -> Self {
        let reference_map = value
            .0
            .iter_mut()
            .map(|(a, entry)| {
                mem::take(&mut entry.refs)
                    .into_iter()
                    .map(|r| (r, a.clone()))
            })
            .flatten()
            .collect();

        let accessor_map = value
            .0
            .into_iter()
            .map(|(a, entry)| (a, entry.value))
            .collect();

        Self {
            reference_map,
            accessor_map
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{self, json};

    use super::*;

    #[test]
    fn serde() {
        let mut map = RefAccessMap::new();
        map.insert("server1", "host1", "data1");
        map.insert("server2", "host2", "data2");
        map.insert_ref("server3", "host2");

        let serialized = serde_json::to_string(&map).expect("Failed to serialize");
        let serialized_json: serde_json::Value =
            serde_json::from_str(&serialized).expect("Failed to parse serialized JSON");

        let expected_json = |host_2_refs| {
            json!({
                "host1": {
                    "value": "data1",
                    "refs": ["server1"]
                },
                "host2": {
                    "value": "data2",
                    "refs": host_2_refs
                }
            })
        };

        let eq = expected_json(["server2", "server3"]) == serialized_json ||
            expected_json(["server3", "server2"]) == serialized_json;
        assert!(eq);

        let deserialized: RefAccessMap<_, _, _> =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(map, deserialized);
    }
}
