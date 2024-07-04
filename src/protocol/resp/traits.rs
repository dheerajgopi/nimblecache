use crate::protocol::resp::types::RespType;
use anyhow::Result;

/// Trait for reading RESP values.
pub trait RespReader {
    /// Reads and return the RESP value.
    async fn read(&mut self) -> Result<Option<RespType>>;
}

/// Trait for writing RESP values.
pub trait RespWriter {
    /// Write the RESP value and return the number of bytes written.
    async fn write(&mut self, resp_data: &RespType) -> Result<usize>;
}
