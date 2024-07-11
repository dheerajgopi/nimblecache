use crate::commands::traits::{CommandBuilder, CommandExecutor};
use crate::protocol::resp::types::RespType;
use bytes::BytesMut;

/// Struct for the PING command.
pub struct Ping {}

impl CommandExecutor for Ping {
    /// Returns a PONG in SimpleString format. Useful for checking if server is alive.
    fn execute(&mut self, _: &[&RespType]) -> (RespType, Option<BytesMut>) {
        return (RespType::SimpleString("PONG".into()), None);
    }
}

impl CommandBuilder for Ping {
    /// Returns a PING command in RESP array format.
    fn build(_: Option<&[&RespType]>) -> RespType {
        RespType::Array(vec![RespType::BulkString("PING".to_string())])
    }
}
