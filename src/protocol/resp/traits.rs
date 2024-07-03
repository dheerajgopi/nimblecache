use crate::protocol::resp::types::RespType;
use anyhow::Result;

pub trait RespReader {
    async fn read(&mut self) -> Result<Option<RespType>>;
}

pub trait RespWriter {
    async fn write(&mut self, resp_data: &RespType) -> Result<usize>;
}