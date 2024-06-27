use anyhow::anyhow;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::protocol::resp::datatypes::DataType;
use crate::protocol::resp::traits::{RespReader, RespWriter};

pub struct Resp2Handler<'a> {
    stream: &'a mut TcpStream,
    buffer: BytesMut,
}

impl<'a> Resp2Handler<'a> {
    pub fn new(stream: &'a mut TcpStream, buffer_cap: usize) -> Self {
        Resp2Handler {
            stream,
            buffer: BytesMut::with_capacity(buffer_cap),
        }
    }
}

impl<'a> RespReader for Resp2Handler<'a> {

    // Parse the RESP data type from the TCP stream.
    async fn read(&mut self) -> anyhow::Result<Option<DataType>> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let parsed_val = DataType::parse(self.buffer.split());

        return match parsed_val {
            Ok(value) => {
                Ok(Some(value.0))
            }
            Err(err) => {
                Err(err)
            }
        };
    }
}

impl<'a> RespWriter for Resp2Handler<'a> {
    async fn write(&mut self, resp_data: &DataType) -> anyhow::Result<usize> {
        let write_data = self.stream.write(resp_data.serialize().as_bytes()).await;
        let bytes_written = match write_data {
            Ok(n) => {
                n
            }
            Err(_) => {
                return Err(anyhow!("Failed to write into TcpStream"));
            }
        };

        Ok(bytes_written)
    }
}
