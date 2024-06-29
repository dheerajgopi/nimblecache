use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::datatypes::DataType;

pub struct Echo {}

impl CommandExecutor for Echo {
    fn execute(&self, args: &[&DataType]) -> DataType {
        if args.len() == 0 {
            return DataType::SimpleError("ERR wrong number of arguments for command".into());
        }

        args[0].clone()
    }
}