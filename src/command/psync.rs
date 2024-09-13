use crate::{replication::Replication, resp::types::RespType};

use super::CommandError;

/// Represents the PSYNC command in Nimblecache.
#[derive(Debug, Clone)]
pub struct Psync {
    replication_id: String,
    offset: Option<u64>,
}

impl Psync {
    pub fn new(replication_id: String, offset: Option<u64>) -> Psync {
        Psync {
            replication_id,
            offset,
        }
    }

    /// Creates a new `Psync` instance from RESP args.
    pub fn with_args(args: Vec<RespType>) -> Result<Psync, CommandError> {
        if args.len() < 2 {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'PSYNC' command",
            )));
        }

        // parse replication id
        let replication_id = &args[0];
        let replication_id = match replication_id {
            RespType::BulkString(id) => id.to_string(),
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. replication id must be a bulk string",
                )));
            }
        };

        // parse offset
        let offset_str = &args[1];
        let offset = match offset_str {
            RespType::BulkString(v) => {
                if v == "-1" {
                    None
                } else {
                    let offset = v.parse::<u64>();
                    match offset {
                        Ok(i) => Some(i),
                        Err(_) => {
                            return Err(CommandError::Other(String::from(
                                "Offset should be an integer",
                            )))
                        }
                    }
                }
            }
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. Value must be in bulk string format",
                )));
            }
        };

        Ok(Psync {
            replication_id,
            offset,
        })
    }

    /// Executes the PSYNC command.
    /// As of now, it always returns FULLRESYNC response.
    ///
    /// # Returns
    ///
    /// Returns a FULLRESYNC response as a `SimpleString`.
    pub fn apply(&self, replication: &Replication) -> RespType {
        let offset = self.offset.map_or("-1".to_string(), |v| v.to_string());

        let mut res = String::from("FULLRESYNC ");
        res.push_str(replication.id.as_str());
        res.push(' ');
        res.push_str(offset.as_str());

        RespType::SimpleString(res)
    }

    pub fn build_command(&self) -> RespType {
        let offset = self.offset.map_or("-1".to_string(), |v| v.to_string());
        RespType::Array(vec![
            RespType::BulkString(String::from("PSYNC")),
            RespType::BulkString(self.replication_id.clone()),
            RespType::BulkString(offset),
        ])
    }
}
