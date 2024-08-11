use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use super::DBError;

/// This struct contains the DB which is shared across all connections.
#[derive(Debug, Clone)]
pub struct Storage {
    db: Arc<DB>,
}

/// This struct holds the data behind a mutex.
#[derive(Debug)]
pub struct DB {
    data: Mutex<Data>,
}

/// The data is stored using a simple HashMap.
#[derive(Debug)]
pub struct Data {
    entries: HashMap<String, Entry>,
}

/// This struct represents the value stored against a key in the database.
#[derive(Debug, Clone)]
pub struct Entry {
    value: Value,
}

/// The type of data stored against a key.
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    List(VecDeque<String>),
}

impl Storage {
    /// Create a new instance of `Storage` which contains the DB.
    pub fn new(db: DB) -> Storage {
        Storage { db: Arc::new(db) }
    }

    /// Get the shared database.
    pub fn db(&self) -> Arc<DB> {
        self.db.clone()
    }
}

impl DB {
    /// Create a new instance of DB.
    pub fn new() -> DB {
        DB {
            data: Mutex::new(Data {
                entries: HashMap::new(),
            }),
        }
    }

    /// Get the string value stored against a key.
    ///
    /// # Arguments
    ///
    /// * `k` - The key on which lookup is performed.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<String>)` - `Some(String)` if key is found in DB, else `None`
    /// * `Err(DBError)` - if key already exists and has non-string data.
    pub fn get(&self, k: &str) -> Result<Option<String>, DBError> {
        let data = match self.data.lock() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let entry = match data.entries.get(k) {
            Some(entry) => entry,
            None => return Ok(None),
        };

        if let Value::String(s) = &entry.value {
            return Ok(Some(s.to_string()));
        }

        Err(DBError::WrongType)
    }

    /// Set a string value against a key.
    ///
    /// # Arguments
    ///
    /// * `k` - The key on which value is to be set.
    ///
    /// * `v` - The value to be set against the key.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If value is successfully added against the key.
    /// * `Err(DBError)` - if key already exists and has non-string data.
    pub fn set(&self, k: String, v: Value) -> Result<(), DBError> {
        let mut data = match self.data.lock() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let entry = match data.entries.get(k.as_str()) {
            Some(entry) => Some(entry),
            None => None,
        };

        if entry.is_some() {
            match entry.unwrap().value {
                Value::String(_) => {}
                _ => return Err(DBError::WrongType),
            }
        }

        data.entries.insert(k.to_string(), Entry::new(v));

        return Ok(());
    }

    /// Adds a new element to the head of a list.
    /// If the key is not present in the DB, and empty list is initialized
    /// against the key before adding the element to the head.
    ///
    /// # Arguments
    ///
    /// * `k` - The key on which list is stored.
    ///
    /// * `v` - The value to be added to the head of the list.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If value is successfully added to the head of the list.
    /// * `Err(DBError)` - if key already exists and has non-list data.
    pub fn lpush(&self, k: String, v: String) -> Result<usize, DBError> {
        let mut data = match self.data.lock() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let entry = match data.entries.get_mut(k.as_str()) {
            Some(entry) => Some(entry),
            None => None,
        };

        match entry {
            Some(e) => {
                let val = &mut e.value;
                match val {
                    Value::List(l) => {
                        l.push_front(v);
                        Ok(l.len())
                    }
                    _ => Err(DBError::WrongType),
                }
            }
            None => {
                let mut list: VecDeque<String> = VecDeque::new();
                list.push_front(v);
                let l_len = list.len();
                data.entries
                    .insert(k.to_string(), Entry::new(Value::List(list)));

                Ok(l_len)
            }
        }
    }

    /// Adds a new element to the tail of a list.
    /// If the key is not present in the DB, and empty list is initialized
    /// against the key before adding the element to the tail.
    ///
    /// # Arguments
    ///
    /// * `k` - The key on which list is stored.
    ///
    /// * `v` - The value to be added to the tail of the list.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If value is successfully added to the tail of the list.
    /// * `Err(DBError)` - if key already exists and has non-list data.
    pub fn rpush(&self, k: String, v: String) -> Result<usize, DBError> {
        let mut data = match self.data.lock() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let entry = match data.entries.get_mut(k.as_str()) {
            Some(entry) => Some(entry),
            None => None,
        };

        match entry {
            Some(e) => {
                let val = &mut e.value;
                match val {
                    Value::List(l) => {
                        l.push_back(v);
                        Ok(l.len())
                    }
                    _ => Err(DBError::WrongType),
                }
            }
            None => {
                let mut list: VecDeque<String> = VecDeque::new();
                list.push_back(v);
                let l_len = list.len();
                data.entries
                    .insert(k.to_string(), Entry::new(Value::List(list)));

                Ok(l_len)
            }
        }
    }
}

impl Entry {
    pub fn new(value: Value) -> Entry {
        Entry { value }
    }
}
