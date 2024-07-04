use std::collections::HashMap;
use std::sync::Mutex;

/// A simple hash-map backed key-value storage for Nimblecache.
/// Synchronizing access to the hash-map is controlled using a mutex.
pub struct SimpleHashMap {
    mem: Mutex<HashMap<String, String>>,
}

impl SimpleHashMap {
    /// Create an instance of SimpleHashMap.
    pub fn new() -> SimpleHashMap {
        SimpleHashMap {
            mem: Mutex::new(HashMap::new()),
        }
    }

    /// Insert/update the value associated to a key.
    pub fn put(&self, k: String, v: String) {
        self.mem.lock().unwrap().insert(k, v);
    }

    /// Return the value associated to a key. If no value is found, a None is returned.
    pub fn get(&self, k: &str) -> Option<String> {
        match self.mem.lock().unwrap().get(k) {
            None => None,
            Some(v) => Some(v.clone()),
        }
    }
}
