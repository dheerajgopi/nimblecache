use std::{
    collections::HashMap,
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
#[derive(Debug)]
pub struct Entry {
    value: Value,
}

/// The type of data stored against a key.
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Other,
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
    /// * `Ok(())` - If value is successfully against the key.
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
                Value::Other => return Err(DBError::WrongType),
            }
        }

        data.entries.insert(k.to_string(), Entry::new(v));

        return Ok(());
    }
}

impl Entry {
    pub fn new(value: Value) -> Entry {
        Entry { value }
    }
}
