use crate::resp::types::RespType;

use super::CommandError;

/// Represents the PING command in Nimblecache.
#[derive(Debug, Clone)]
pub struct Ping {
    /// Custom message
    msg: Option<String>,
}

impl Ping {
    /// Creates a new `Ping` instance from the given arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `RespType` representing the arguments to the PING command.
    ///
    /// # Returns
    ///
    /// * `Ok(Ping)` if parsing succeeds.
    /// * `Err(CommandError)` if parsing fails.
    pub fn with_args(args: Vec<RespType>) -> Result<Ping, CommandError> {
        if args.len() == 0 {
            return Ok(Ping { msg: None });
        }

        let msg = match &args[0] {
            RespType::BulkString(s) => s.clone(),
            _ => return Err(CommandError::Other(String::from("Invalid message"))),
        };

        Ok(Ping { msg: Some(msg) })
    }

    /// Executes the PING command.
    ///
    /// # Returns
    ///
    /// A `RespType` representing the response:
    /// - If no message was provided, it returns "PONG" as a `SimpleString`.
    /// - If a message was provided, it returns that message as a `BulkString`.
    pub fn apply(&self) -> RespType {
        if let Some(msg) = &self.msg {
            RespType::BulkString(msg.to_string())
        } else {
            RespType::SimpleString(String::from("PONG"))
        }
    }
}
