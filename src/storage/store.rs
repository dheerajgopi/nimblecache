use crate::storage::simplemap::SimpleHashMap;
use crate::storage::store::Store::SimpleMap;

/// This enum is a wrapper for the different data-stores supported by Nimblecache.
pub enum Store {
    /// A simple hash-map based key-value store.
    SimpleMap(SimpleHashMap),
}

impl Store {
    /// Creates a [Self::SimpleMap] data-store
    pub fn new_simple_map() -> Store {
        SimpleMap(SimpleHashMap::new())
    }

    /// Insert/update the value associated to a key.
    pub fn put(&self, k: String, v: String) {
        match self {
            SimpleMap(mem) => mem.put(k, v),
        }
    }

    /// Return the value associated to a key. If no value is found, a None is returned.
    pub fn get(&self, k: &str) -> Option<String> {
        match self {
            SimpleMap(mem) => mem.get(k),
        }
    }
}
