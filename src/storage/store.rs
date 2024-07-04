use crate::storage::simplemap::SimpleHashMap;
use crate::storage::store::Store::SimpleMap;

pub enum Store {
    SimpleMap(SimpleHashMap),
}

impl Store {
    pub fn new_simple_map() -> Store {
        SimpleMap(SimpleHashMap::new())
    }

    pub fn put(&self, k: String, v: String) {
        match self {
            SimpleMap(mem) => mem.put(k, v),
        }
    }

    pub fn get(&self, k: &str) -> Option<String> {
        match self {
            SimpleMap(mem) => mem.get(k),
        }
    }
}
