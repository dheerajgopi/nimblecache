use crate::protocol::resp::datatypes::DataType;

pub trait CommandExecutor {
    fn execute(&self, args: &[&DataType]) -> DataType;
}