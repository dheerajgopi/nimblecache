use crate::protocol::resp::types::RespType;

pub trait CommandExecutor {
    fn execute(&mut self, args: &[&RespType]) -> RespType;
}