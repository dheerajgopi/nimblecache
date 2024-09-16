use crate::{resp::types::RespType, storage::db::DB};

use super::CommandError;

/// Represents the DEL command in Nimblecache.
#[derive(Debug, Clone)]
pub struct Del {
    keys: Vec<String>,
}

impl Del {
    /// Creates a new `Del` instance from the given arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `RespType` representing the arguments to the DEL command.
    ///
    /// # Returns
    ///
    /// * `Ok(Del)` if parsing succeeds.
    /// * `Err(CommandError)` if parsing fails.
    pub fn with_args(args: Vec<RespType>) -> Result<Del, CommandError> {
        if args.is_empty() {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'DEL' command",
            )));
        }

        let mut keys: Vec<String> = vec![];
        for key in args.iter() {
            // validate if all keys are BulkStrings
            if let RespType::BulkString(k) = key {
                keys.push(k.clone());
            } else {
                return Err(CommandError::Other(
                    "Invalid argument. Key must be a bulk string".to_string(),
                ));
            }
        }

        Ok(Del { keys })
    }

    /// Executes the DEL command.
    ///
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// # Returns
    ///
    /// It returns the number of deleted keys as an `Integer` if keys are successfully deleted.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.bulk_del(&self.keys.iter().map(AsRef::as_ref).collect::<Vec<&str>>()) {
            Ok(del_count) => RespType::Integer(del_count as i64),
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }
}
