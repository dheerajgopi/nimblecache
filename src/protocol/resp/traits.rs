use crate::protocol::resp::types::RespType;
use anyhow::Result;
use bytes::BytesMut;

/// Trait for reading RESP values.
pub trait RespReader {
    /// Reads and return the RESP value.
    /// Optional bytes payload will have some value only in special cases
    /// like PSYNC.
    async fn read(&mut self) -> Result<(Option<RespType>, Option<BytesMut>)>;
}

/// Trait for writing RESP values.
pub trait RespWriter {
    /// Write the RESP value and return the number of bytes written.
    async fn write(&mut self, resp_data: &RespType) -> Result<usize>;
}

/// Trait for writing byte values.
pub trait BytesWriter {
    /// Write the byte values and return the number of bytes written.
    async fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize>;
}
