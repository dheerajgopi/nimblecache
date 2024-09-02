// src/storage/db.rs

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::DBError;

/// The Storage struct is designed to act as a wrapper around the core database,
/// allowing it to be shared across multiple connections. The database is encapsulated within an Arc,
/// to enable concurrent access.
#[derive(Debug, Clone)]
pub struct Storage {
    db: Arc<DB>,
}

/// The DB struct is the component that houses the actual data,
/// which is stored in a RwLock wrapped around a HashMap. This ensures thread-safe read and write operations.
#[derive(Debug)]
pub struct DB {
    data: RwLock<HashMap<String, Entry>>,
}

/// The Entry struct represents the value associated with a particular key in the database.
/// This struct encapsulates the Value enum, which allows for different types of data to be stored.
#[derive(Debug, Clone)]
pub struct Entry {
    value: Value,
}

/// The `Value` enum allows for storing various types of data associated with a key.
/// Currently, it supports only String data type. But it can be expanded in the future
/// to support more data types as needed (like List, Hash etc).
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
}

impl Storage {
    /// Create a new instance of `Storage` which contains the DB.
    pub fn new(db: DB) -> Storage {
        Storage { db: Arc::new(db) }
    }

    /// Returns a clone of the shared database (`Arc<DB>`).
    ///
    /// This method provides access to the underlying database, which is shared across all
    /// connections. The database is wrapped in an `Arc` to ensure concurrent access by multiple threads.
    pub fn db(&self) -> Arc<DB> {
        self.db.clone()
    }
}

impl DB {
    /// Create a new instance of DB.
    pub fn new() -> DB {
        DB {
            data: RwLock::new(HashMap::new()),
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
        let data = match self.data.read() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let entry = match data.get(k) {
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
        let mut data = match self.data.write() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let entry = match data.get(k.as_str()) {
            Some(entry) => Some(entry),
            None => None,
        };

        if entry.is_some() {
            match entry.unwrap().value {
                Value::String(_) => {}
                _ => return Err(DBError::WrongType),
            }
        }

        data.insert(k.to_string(), Entry::new(v));

        return Ok(());
    }
}

impl Entry {
    pub fn new(value: Value) -> Entry {
        Entry { value }
    }
}
