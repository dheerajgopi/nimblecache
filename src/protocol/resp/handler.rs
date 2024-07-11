use crate::protocol::resp::traits::{BytesWriter, RespReader, RespWriter};
use crate::protocol::resp::types::RespType;
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// RespHandler can read RESP values from a TcpStream and write RESP values into the same TcpStream.
pub struct RespHandler<'a> {
    stream: &'a mut TcpStream,
    buffer: BytesMut,
}

impl<'a> RespHandler<'a> {
    /// Creates a new RespHandler using the given TcpStream. The internal buffer capacity can
    /// also be specified using `buffer_cap` argument.
    pub fn new(stream: &'a mut TcpStream, buffer_cap: usize) -> Self {
        RespHandler {
            stream,
            buffer: BytesMut::with_capacity(buffer_cap),
        }
    }
}

impl<'a> RespReader for RespHandler<'a> {
    /// Parse the RESP value from the TcpStream.
    /// Bytes payload will have some value only in special cases
    /// like PSYNC.
    async fn read(&mut self) -> Result<(Option<RespType>, Option<BytesMut>)> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;

        if bytes_read == 0 {
            return Ok((None, None));
        }

        let mut buf = self.buffer.split();
        let parsed_val = RespType::parse(buf.clone());

        return match parsed_val {
            Ok((value, bytes_read)) => {
                if bytes_read < buf.len() {
                    return Ok((Some(value), Some(buf.split_off(bytes_read))));
                }
                Ok((Some(value), None))
            }
            Err(err) => Err(err),
        };
    }
}

impl<'a> RespWriter for RespHandler<'a> {
    /// Write the RESP value into the TcpStream and return the number of bytes written.
    async fn write(&mut self, resp_data: &RespType) -> Result<usize> {
        let write_data = self.stream.write(resp_data.serialize().as_bytes()).await;
        let bytes_written = match write_data {
            Ok(n) => n,
            Err(_) => {
                return Err(anyhow!("Failed to write into TcpStream"));
            }
        };

        Ok(bytes_written)
    }
}

impl<'a> BytesWriter for RespHandler<'a> {
    /// Write the byte values into the TcpStream and return the number of bytes written.
    async fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize> {
        match self.stream.write(bytes).await {
            Ok(b) => Ok(b),
            Err(e) => Err(anyhow!("Error writing to TCP stream: {}", e)),
        }
    }
}
