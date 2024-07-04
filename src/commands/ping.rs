use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;

pub struct Ping {}

impl CommandExecutor for Ping {
    fn execute(&mut self, _: &[&RespType]) -> RespType {
        return RespType::SimpleString("PONG".into());
    }
}
