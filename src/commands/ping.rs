use crate::commands::traits::{CommandBuilder, CommandExecutor};
use crate::protocol::resp::types::RespType;

/// Struct for the PING command.
pub struct Ping {}

impl CommandExecutor for Ping {
    /// Returns a PONG in SimpleString format. Useful for checking if server is alive.
    fn execute(&mut self, _: &[&RespType]) -> RespType {
        return RespType::SimpleString("PONG".into());
    }
}

impl CommandBuilder for Ping {
    fn build(_: Option<&[&RespType]>) -> RespType {
        RespType::Array(vec![RespType::BulkString("PING".to_string())])
    }
}
