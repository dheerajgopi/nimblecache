// src/command/lpush.rs

use crate::{resp::types::RespType, storage::db::DB};

use super::CommandError;

/// Represents the LPUSH command in Nimblecache.
#[derive(Debug, Clone)]
pub struct LPush {
    key: String,
    values: Vec<String>,
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
    /// * `Ok(LPush)` if parsing succeeds.
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

        // parse values
        let mut values: Vec<String> = vec![];
        for arg in args[1..].iter() {
            match arg {
                RespType::BulkString(v) => values.push(v.to_string()),
                _ => {
                    return Err(CommandError::Other(String::from(
                        "Invalid argument. Value must be a bulk string",
                    )));
                }
            }
        }

        Ok(LPush {
            key: key.to_string(),
            values,
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
    /// It returns the length of the list if value is successfully written.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.lpush(self.key.clone(), self.values.clone()) {
            Ok(len) => RespType::Integer(len as i64),
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }

    pub fn build_command(&self) -> RespType {
        let mut args: Vec<RespType> = vec![
            RespType::BulkString(String::from("LPUSH")),
            RespType::BulkString(self.key.clone()),
        ];

        let arg_vals = self.values.clone();
        for arg in arg_vals.iter() {
            args.push(RespType::BulkString(arg.to_string()));
        }

        RespType::Array(args)
    }
}
