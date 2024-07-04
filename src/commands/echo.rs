use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;

pub struct Echo {}

impl CommandExecutor for Echo {
    fn execute(&mut self, args: &[&RespType]) -> RespType {
        if args.len() == 0 {
            return RespType::SimpleError("ERR wrong number of arguments for command".into());
        }

        args[0].clone()
    }
}
