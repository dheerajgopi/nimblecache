use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
    hash::Hash,
    sync::{Arc, RwLock},
};

use log::error;
use time::OffsetDateTime;
use tokio::sync::broadcast::{self, Receiver, Sender};

use super::{DBError, DBEvent};

/// This struct contains the DB which is shared across all connections.
#[derive(Debug, Clone)]
pub struct Storage {
    db: Arc<DB>,
}

/// This struct holds the data behind a RwLock.
#[derive(Debug)]
pub struct DB {
    data: RwLock<HashMap<Key, Entry>>,
    events: Arc<Sender<DBEvent>>,
}

/// This struct represents the key in the database. It encloses the value for
/// the key its expiry (optional).
#[derive(Debug, Clone)]
pub struct Key {
    value: String,
    expiry: Option<OffsetDateTime>,
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
        let (tx, _) = broadcast::channel(1024);

        DB {
            data: RwLock::new(HashMap::new()),
            events: Arc::new(tx),
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
    pub fn get(&self, k: String) -> Result<Option<String>, DBError> {
        let data = match self.data.read() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let (_, entry) = match data.get_key_value(&k.into()) {
            Some(pair) => pair,
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
    /// * `expiry_ts` (optional)- Time at which key expires.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If value is successfully added against the key.
    /// * `Err(DBError)` - if key already exists and has non-string data.
    pub fn set(
        &self,
        k: String,
        v: Value,
        expiry_ts: Option<OffsetDateTime>,
    ) -> Result<(), DBError> {
        let mut data = match self.data.write() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let key = Key::new(k.clone(), expiry_ts);
        let existing_kv_pair = data.get_key_value(&key);

        if let Some((_, entry)) = existing_kv_pair {
            match entry.value {
                Value::String(_) => {}
                _ => return Err(DBError::WrongType),
            }
        }

        data.insert(key, Entry::new(v));

        if let Some(expiry) = expiry_ts {
            let key = k.clone();
            let evt = DBEvent::SetKeyExpiry((expiry, key));

            if let Err(e) = self.events.send(evt) {
                error!("Failed to send set expiry event: {}", e);
                return Err(DBError::Other(e.to_string()));
            }
        }

        Ok(())
    }

    /// Add new elements to the head of a list.
    /// If the key is not present in the DB, and empty list is initialized
    /// against the key before adding the elements to the head.
    ///
    /// # Arguments
    ///
    /// * `k` - The key on which list is stored.
    ///
    /// * `v` - The values to be added to the head of the list.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If values are added successfully to the head of the list.
    /// * `Err(DBError)` - if key already exists and has non-list data.
    pub fn lpush(&self, k: String, v: Vec<String>) -> Result<usize, DBError> {
        let mut data = match self.data.write() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let key: Key = k.into();
        let entry = data.get_mut(&key);

        match entry {
            Some(e) => {
                let val = &mut e.value;
                match val {
                    Value::List(l) => {
                        for each in v.iter().cloned() {
                            l.push_front(each);
                        }
                        Ok(l.len())
                    }
                    _ => Err(DBError::WrongType),
                }
            }
            None => {
                let list = VecDeque::from(v);
                let l_len = list.len();
                data.insert(key, Entry::new(Value::List(list)));

                Ok(l_len)
            }
        }
    }

    /// Adds new elements to the tail of a list.
    /// If the key is not present in the DB, and empty list is initialized
    /// against the key before adding the elements to the tail.
    ///
    /// # Arguments
    ///
    /// * `k` - The key on which list is stored.
    ///
    /// * `v` - The values to be added to the tail of the list.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If value are added successfully to the tail of the list.
    /// * `Err(DBError)` - if key already exists and has non-list data.
    pub fn rpush(&self, k: String, v: Vec<String>) -> Result<usize, DBError> {
        let mut data = match self.data.write() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let key: Key = k.into();
        let entry = data.get_mut(&key);

        match entry {
            Some(e) => {
                let val = &mut e.value;
                match val {
                    Value::List(l) => {
                        for each in v.iter().cloned() {
                            l.push_back(each);
                        }
                        Ok(l.len())
                    }
                    _ => Err(DBError::WrongType),
                }
            }
            None => {
                let list = VecDeque::from(v);
                let l_len = list.len();
                data.insert(key, Entry::new(Value::List(list)));

                Ok(l_len)
            }
        }
    }

    /// Returns the specified number of elements of the list stored at key, based on the start and stop indices.
    /// These offsets can also be negative numbers indicating offsets starting at the end of the list.
    /// For example, -1 is the last element of the list, -2 the penultimate, and so on.
    /// Please note that the item at stop index is also included in the result.
    ///
    /// If the specified key is not found, an empty list is returned.
    ///
    /// # Arguments
    ///
    /// * `k` - The key on which list is stored.
    ///
    /// * `start_idx` - The start index.
    ///
    /// * `stop_idx` - The end index.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - If values are retrieved successfully from the list.
    /// * `Err(DBError)` - if key already exists and has non-list data.
    pub fn lrange(&self, k: String, start_idx: i64, stop_idx: i64) -> Result<Vec<String>, DBError> {
        let data = match self.data.read() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let key: Key = k.into();
        let entry = match data.get(&key) {
            Some(entry) => entry,
            None => return Ok(vec![]),
        };

        match &entry.value {
            Value::List(l) => {
                let l_len = l.len() as i64;
                let (rounded_start_idx, rounded_stop_idx) =
                    Self::round_list_indices(l_len, start_idx, stop_idx);
                Ok(l.range(rounded_start_idx..rounded_stop_idx)
                    .cloned()
                    .collect())
            }
            _ => Err(DBError::WrongType),
        }
    }

    /// Delete a key from the DB and return an Option containing its value.
    ///
    /// # Arguments
    ///
    /// * `k` - The key to be deleted.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Entry>)` - An Option containing the value stored against the deleted key.
    /// * `Err(DBError)` - if key deletion fails.
    pub fn del(&self, k: &str) -> Result<Option<Entry>, DBError> {
        let mut data = match self.data.write() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        Ok(data.remove(&k.into()))
    }

    /// Delete a list of keys from the DB and return the number of keys deleted.
    /// Keys which does not exist in the DB are not considered towards the deleted-keys count.
    ///
    /// # Arguments
    ///
    /// * `keys` - The list of keys to be deleted.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of keys deleted (which were present in the DB).
    /// * `Err(DBError)` - if key deletion fails.
    pub fn bulk_del(&self, keys: &[&str]) -> Result<usize, DBError> {
        let mut data = match self.data.write() {
            Ok(data) => data,
            Err(e) => return Err(DBError::Other(format!("{}", e))),
        };

        let mut del_count: usize = 0;
        let mut del_keys_with_expiry: Vec<(OffsetDateTime, String)> = vec![];

        for k in keys {
            let key = Key::from(*k);
            let kv_pair = data.remove_entry(&key);
            if let Some((k, _)) = kv_pair {
                del_count += 1;

                if let Some(expiry_ts) = k.expiry {
                    del_keys_with_expiry.push((expiry_ts, k.value));
                }
            }
        }

        if !del_keys_with_expiry.is_empty() {
            if let Err(e) = self.events.send(DBEvent::BulkDelKeys(del_keys_with_expiry)) {
                error!("Failed to send bulk key deletion event: {}", e);
                return Err(DBError::Other(e.to_string()));
            }
        }

        Ok(del_count)
    }

    pub fn subscribe_events(&self) -> Receiver<DBEvent> {
        self.events.subscribe()
    }

    /// Round index to 0, if the given index value is less than zero.
    /// Round index to list length, if the given index value is greater then the list length.
    fn round_list_index(list_len: i64, idx: i64) -> usize {
        if idx < 0 {
            let idx = list_len - idx.abs();
            if idx < 0 {
                return 0;
            } else {
                return idx as usize;
            }
        }

        if idx >= list_len {
            return (list_len - 1) as usize;
        }

        idx as usize
    }

    /// Round the start and stop indices using `Self::round_list_index` method and return them as
    /// a tuple.
    /// Special condition: If stop index is lower than start index, return (0, 0).
    fn round_list_indices(list_len: i64, start_idx: i64, stop_idx: i64) -> (usize, usize) {
        if stop_idx < start_idx {
            return (0, 0);
        }

        let rounded_start_idx = Self::round_list_index(list_len, start_idx);
        let rounded_stop_idx = Self::round_list_index(list_len, stop_idx);

        match rounded_start_idx.cmp(&rounded_stop_idx) {
            std::cmp::Ordering::Less => (rounded_start_idx, rounded_stop_idx + 1),
            std::cmp::Ordering::Equal => (rounded_start_idx, rounded_start_idx + 1),
            std::cmp::Ordering::Greater => (0, 0),
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<String> for Key {
    fn from(value: String) -> Self {
        Key {
            value,
            expiry: None,
        }
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Key {
            value: s.to_string(),
            expiry: None,
        }
    }
}

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Key {}

impl Key {
    pub fn new(value: String, expiry: Option<OffsetDateTime>) -> Key {
        Key { value, expiry }
    }
}

impl Entry {
    pub fn new(value: Value) -> Entry {
        Entry { value }
    }
}
