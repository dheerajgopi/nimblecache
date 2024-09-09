// src/command/lrange.rs

use crate::{resp::types::RespType, storage::db::DB};

use super::CommandError;

/// Represents the LRANGE command in Nimblecache.
#[derive(Debug, Clone)]
pub struct LRange {
    key: String,
    start_idx: i64,
    end_idx: i64,
}

impl LRange {
    /// Creates a new `LRANGE` instance from the given arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `RespType` representing the arguments to the SET command.
    ///
    /// # Returns
    ///
    /// * `Ok(LRange)` if parsing succeeds.
    /// * `Err(CommandError)` if parsing fails.
    pub fn with_args(args: Vec<RespType>) -> Result<LRange, CommandError> {
        if args.len() < 3 {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'LRANGE' command",
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

        // parse start index
        let value = &args[1];
        let start_idx = match value {
            RespType::BulkString(v) => {
                let start_idx = v.parse::<i64>();
                match start_idx {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(CommandError::Other(String::from(
                            "Start index should be an integer",
                        )))
                    }
                }
            }
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. Value must be an integer in bulk string format",
                )));
            }
        };

        // parse end index
        let value = &args[2];
        let end_idx = match value {
            RespType::BulkString(v) => {
                let end_idx = v.parse::<i64>();
                match end_idx {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(CommandError::Other(String::from(
                            "End index should be an integer",
                        )))
                    }
                }
            }
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. Value must be an integer in bulk string format",
                )));
            }
        };

        Ok(LRange {
            key: key.to_string(),
            start_idx,
            end_idx,
        })
    }

    /// Executes the LRANGE command.
    ///
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// # Returns
    ///
    /// It returns the specified number of elements in the list stored at key, based on start and stop indices.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.lrange(self.key.clone(), self.start_idx, self.end_idx) {
            Ok(elems) => {
                let sub_list = elems
                    .iter()
                    .cloned()
                    .map(|e| RespType::BulkString(e))
                    .collect();
                RespType::Array(sub_list)
            }
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }
}
