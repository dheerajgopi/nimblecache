pub mod frame;
pub mod types;

/// Represents errors that can occur during RESP parsing.
#[derive(Debug)]
pub enum RespError {
    /// Represents an error in parsing a bulk string, with an error message.
    InvalidBulkString(String),
    /// Represents an error in parsing a simple string, with an error message.
    InvalidSimpleString(String),
    /// Represents an error in parsing an array, with an error message.
    InvalidArray(String),
    /// Represents any other error with a descriptive message.
    Other(String),
}

impl std::fmt::Display for RespError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RespError::Other(msg) => msg.as_str().fmt(f),
            RespError::InvalidBulkString(msg) => msg.as_str().fmt(f),
            RespError::InvalidSimpleString(msg) => msg.as_str().fmt(f),
            RespError::InvalidArray(msg) => msg.as_str().fmt(f),
        }
    }
}
