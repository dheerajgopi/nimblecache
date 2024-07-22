use crate::commands::traits::{CommandBuilder, CommandExecutor, CommandHandler};
use crate::protocol::resp::types::RespType;
use bytes::BytesMut;
use tokio::net::TcpStream;

/// Struct for the REPLCONF command.
pub struct Replconf<'a> {
    stream: &'a mut TcpStream,
}

impl<'a> Replconf<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Replconf<'a> {
        Replconf { stream }
    }
}

impl<'a> CommandExecutor for Replconf<'a> {
    /// Returns an OK for now.
    /// TODO: Actual replication configuration
    fn execute(&self, _: &[&RespType]) -> (RespType, Option<BytesMut>) {
        return (RespType::SimpleString("OK".into()), None);
    }
}

impl<'a> CommandHandler for Replconf<'a> {
    /// Execute the REPLCONF command, and then write the output to the response TCP stream.
    async fn handle(&mut self, args: &[&RespType]) -> anyhow::Result<usize> {
        let (res, _) = self.execute(args);
        RespType::write_to_stream(self.stream, &res).await
    }
}

impl<'a> CommandBuilder for Replconf<'a> {
    /// Returns a REPLCONF command in RESP array format.
    fn build(args: Option<&[&RespType]>) -> RespType {
        let mut cmd = vec![RespType::BulkString("REPLCONF".to_string())];
        if args.is_some() {
            let cmd_args = args
                .unwrap()
                .iter()
                .map(|&r| r.clone())
                .collect::<Vec<RespType>>();
            cmd.extend(cmd_args);
        }

        RespType::Array(cmd)
    }
}
