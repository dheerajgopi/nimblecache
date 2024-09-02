// src/command/set.rs

use crate::{
    resp::types::RespType,
    storage::db::{Value, DB},
};

use super::CommandError;

/// Represents the SET command in Nimblecache.
///
/// The `Set` struct encapsulates the key-value pair for the SET command, which is used
/// to store a string value against a key in the database.
#[derive(Debug, Clone)]
pub struct Set {
    key: String,
    value: String,
}

impl Set {
    /// Creates a new `Set` instance from the given arguments.
    ///
    /// This function parses the arguments provided in the form of a `RespType` vector.
    /// It validates and extracts the key and value for the SET command.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `RespType` representing the arguments to the SET command.
    ///
    /// # Returns
    ///
    /// * `Ok(Set)` - If parsing succeeds and the key-value pair is valid.
    /// * `Err(CommandError)` - if parsing fails due to validation errors.
    pub fn with_args(args: Vec<RespType>) -> Result<Set, CommandError> {
        if args.len() < 2 {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'SET' command",
            )));
        }

        // parse key
        let key = &args[0];
        let key = match key {
            RespType::BulkString(k) => k,
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. Key must be a bulk string",
                )));
            }
        };

        // parse value
        let value = &args[1];
        let value = match value {
            RespType::BulkString(v) => v.to_string(),
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. Value must be a bulk string",
                )));
            }
        };

        Ok(Set {
            key: key.to_string(),
            value,
        })
    }

    /// Executes the SET command.
    ///
    /// This method writes the string value to the database under the specified key.
    /// If the operation is successful, it returns an "OK" response as a `BulkString`.
    /// If the operation fails, it returns an error response.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the `DB` instance where the key-value pair should be stored.
    ///
    /// # Returns
    ///
    /// * `BulkString("OK")` - If the value is successfully written.
    /// * `SimpleError` - If the operation fails due to some error.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.set(self.key.clone(), Value::String(self.value.clone())) {
            Ok(_) => RespType::BulkString("OK".to_string()),
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }
}
