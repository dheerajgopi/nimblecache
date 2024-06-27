use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::datatypes::DataType;
use anyhow::Result;

pub struct Ping {}

impl CommandExecutor for Ping {
    fn execute(&self, _: &[&DataType]) -> Result<DataType> {
        return Ok(DataType::SimpleString("PONG".into()));
    }
}