use std::collections::HashMap;
use std::sync::Mutex;

pub struct SimpleHashMap {
    mem: Mutex<HashMap<String, String>>
}

impl SimpleHashMap {
    pub fn new() -> SimpleHashMap {
        SimpleHashMap {
            mem: Mutex::new(HashMap::new())
        }
    }

    pub fn put(&self, k: String, v: String) {
        self.mem.lock().unwrap().insert(k, v);
        // self.mem.insert(k, v);
    }

    pub fn get(&self, k: &str) -> Option<String> {
        match self.mem.lock().unwrap().get(k) {
            None => {
                None
            }
            Some(v) => {
                Some(v.clone())
            }
        }
    }
}