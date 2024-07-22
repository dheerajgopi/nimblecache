use crate::commands::traits::{CommandBuilder, CommandExecutor, CommandHandler};
use crate::protocol::resp::types::RespType;
use bytes::BytesMut;
use tokio::net::TcpStream;

/// Struct for the PING command.
pub struct Ping<'a> {
    stream: &'a mut TcpStream,
}

impl<'a> Ping<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Ping<'a> {
        Ping { stream }
    }
}

impl<'a> CommandExecutor for Ping<'a> {
    /// Returns a PONG in SimpleString format. Useful for checking if server is alive.
    fn execute(&self, _: &[&RespType]) -> (RespType, Option<BytesMut>) {
        return (RespType::SimpleString("PONG".into()), None);
    }
}

impl<'a> CommandHandler for Ping<'a> {
    /// Execute the PING command, and then write the output to the response TCP stream.
    async fn handle(&mut self, args: &[&RespType]) -> anyhow::Result<usize> {
        let (res, _) = self.execute(args);
        RespType::write_to_stream(self.stream, &res).await
    }
}

impl<'a> CommandBuilder for Ping<'a> {
    /// Returns a PING command in RESP array format.
    fn build(_: Option<&[&RespType]>) -> RespType {
        RespType::Array(vec![RespType::BulkString("PING".to_string())])
    }
}
