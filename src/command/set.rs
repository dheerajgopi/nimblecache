use crate::{
    resp::types::RespType,
    storage::db::{Value, DB},
};

use super::CommandError;

/// Represents the SET command in Nimblecache.
#[derive(Debug, Clone)]
pub struct Set {
    key: String,
    value: String,
}

impl Set {
    /// Creates a new `Set` instance from the given arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `RespType` representing the arguments to the SET command.
    ///
    /// # Returns
    ///
    /// * `Ok(Set)` if parsing succeeds.
    /// * `Err(CommandError)` if parsing fails.
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
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// # Returns
    ///
    /// It returns an 'OK` as a `BulkString` if value is successfully written.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.set(self.key.clone(), Value::String(self.value.clone())) {
            Ok(_) => RespType::BulkString("OK".to_string()),
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }
}
