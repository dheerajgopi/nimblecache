use crate::{resp::types::RespType, storage::db::DB};

use super::CommandError;

/// Represents the GET command in Nimblecache.
#[derive(Debug, Clone)]
pub struct Get {
    /// Key to be searched in the database
    key: String,
}

impl Get {
    /// Creates a new `Get` instance from the given arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `RespType` representing the arguments to the GET command.
    ///
    /// # Returns
    ///
    /// * `Ok(Get)` if parsing succeeds.
    /// * `Err(CommandError)` if parsing fails.
    pub fn with_args(args: Vec<RespType>) -> Result<Get, CommandError> {
        if args.is_empty() {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'GET' command",
            )));
        }

        // parse key
        let key = &args[0];
        let key = match key {
            RespType::BulkString(k) => k.to_string(),
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. Key must be a bulk string",
                )));
            }
        };

        Ok(Get { key })
    }

    /// Executes the GET command.
    ///
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// # Returns
    ///
    /// - If key is present in DB - Value of the key as a `BulkString`
    /// - If key is not found in DB - A `NullBulkString`
    /// - If an error is encountered - A `SimpleError` with an error message
    pub fn apply(&self, db: &DB) -> RespType {
        match db.get(self.key.clone()) {
            Ok(val) => match val {
                Some(s) => RespType::BulkString(s),
                None => RespType::NullBulkString,
            },
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }
}
