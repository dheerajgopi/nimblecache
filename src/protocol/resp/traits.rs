use crate::protocol::resp::datatypes::DataType;
use anyhow::Result;

pub trait RespReader {
    async fn read(&mut self) -> Result<Option<DataType>>;
}

pub trait RespWriter {
    async fn write(&mut self, resp_data: &DataType) -> Result<usize>;
}