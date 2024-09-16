use time::OffsetDateTime;

pub mod db;
pub mod ttl;

/// Represents database events that can occur in the system.
#[derive(Debug, Clone)]
pub enum DBEvent {
    /// Event triggered when a key's expiry time is set.
    ///
    /// Contains a tuple with:
    /// - `OffsetDateTime`: The expiration time for the key.
    /// - `String`: The key for which the expiry is set.
    SetKeyExpiry((OffsetDateTime, String)),
    /// Event triggered when a list of keys are deleted from DB.
    ///
    /// Contains the list of keys as a vector of tuples. Each item in the tuple contains:
    /// - `OffsetDateTime`: The expiration time for the key.
    /// - `String`: The key for which the expiry is set.
    BulkDelKeys(Vec<(OffsetDateTime, String)>),
}

/// Represents errors that can occur during DB operations.
#[derive(Debug)]
pub enum DBError {
    /// Represents an error where wrong data type is encountered against a key.
    WrongType,
    /// Represents any other error with a descriptive message.
    Other(String),
}

impl std::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::WrongType => {
                "WRONGTYPE Operation against a key holding the wrong kind of value".fmt(f)
            }
            DBError::Other(msg) => msg.as_str().fmt(f),
        }
    }
}
