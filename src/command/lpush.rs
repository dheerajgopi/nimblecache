use crate::{resp::types::RespType, storage::db::DB};

use super::CommandError;

/// Represents the LPUSH command in Nimblecache.
#[derive(Debug, Clone)]
pub struct LPush {
    key: String,
    value: String,
}

impl LPush {
    /// Creates a new `LPUSH` instance from the given arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `RespType` representing the arguments to the SET command.
    ///
    /// # Returns
    ///
    /// * `Ok(Set)` if parsing succeeds.
    /// * `Err(CommandError)` if parsing fails.
    pub fn with_args(args: Vec<RespType>) -> Result<LPush, CommandError> {
        if args.len() < 2 {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'LPUSH' command",
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

        Ok(LPush {
            key: key.to_string(),
            value,
        })
    }

    /// Executes the LPUSH command.
    ///
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// # Returns
    ///
    /// It returns an 'OK` as a `BulkString` if value is successfully written.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.lpush(self.key.clone(), self.value.clone()) {
            Ok(len) => RespType::Integer(len as i64),
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }
}
