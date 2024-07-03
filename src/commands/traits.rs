use crate::protocol::resp::types::RespType;

pub trait CommandExecutor {
    fn execute(&self, args: &[&RespType]) -> RespType;
}