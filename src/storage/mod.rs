pub mod db;

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
