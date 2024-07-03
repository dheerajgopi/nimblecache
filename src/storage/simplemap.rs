use std::collections::HashMap;

pub struct SimpleHashMap {
    mem: HashMap<String, String>
}

impl SimpleHashMap {
    pub fn new() -> SimpleHashMap {
        SimpleHashMap {
            mem: HashMap::new()
        }
    }

    pub fn put(&mut self, k: String, v: String) {
        self.mem.insert(k, v);
    }

    pub fn get(& self, k: &str) -> Option<&str> {
        match self.mem.get(k) {
            None => {
                None
            }
            Some(v) => {
                Some(v.as_str())
            }
        }
    }
}