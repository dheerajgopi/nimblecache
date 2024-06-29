use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::datatypes::DataType;

pub struct Ping {}

impl CommandExecutor for Ping {
    fn execute(&self, _: &[&DataType]) -> DataType {
        return DataType::SimpleString("PONG".into());
    }
}