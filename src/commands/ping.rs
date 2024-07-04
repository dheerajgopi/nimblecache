use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;

/// Struct for the PING command.
pub struct Ping {}

impl CommandExecutor for Ping {
    /// Returns a PONG in SimpleString format. Useful for checking if server is alive.
    fn execute(&mut self, _: &[&RespType]) -> RespType {
        return RespType::SimpleString("PONG".into());
    }
}
