use crate::resp::types::RespType;

use super::CommandError;

/// Represents the INFO command in Nimblecache.
#[derive(Debug, Clone)]
pub struct Info {}

impl Info {
    /// Creates a new `Info` instance from the given arguments.
    ///
    /// # Returns
    ///
    /// * `Ok(Info)` if parsing succeeds.
    /// * `Err(CommandError)` if parsing fails.
    pub fn with_args(_: Vec<RespType>) -> Result<Info, CommandError> {
        Ok(Info {})
    }

    /// Executes the INFO command.
    ///
    /// # Returns
    ///
    /// Returns a `BulkString` with server info.
    pub fn apply(&self) -> RespType {
        RespType::BulkString(String::from("role:master\n"))
    }
}
