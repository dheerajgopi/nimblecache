use crate::protocol::resp::datatypes::DataType;
use anyhow::Result;

pub trait CommandExecutor {
    fn execute(&self, args: &[&DataType]) -> Result<DataType>;
}