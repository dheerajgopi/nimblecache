use crate::commands::traits::{CommandExecutor, CommandHandler};
use crate::protocol::resp::types::RespType;
use bytes::BytesMut;
use tokio::net::TcpStream;

/// Struct for the ECHO command.
pub struct Echo<'a> {
    stream: &'a mut TcpStream,
}

impl<'a> Echo<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Echo<'a> {
        Echo { stream }
    }
}

impl<'a> CommandExecutor for Echo<'a> {
    /// Returns the given argument as it is. In other words, it simply echoes back the given argument.
    ///
    /// # Validations
    /// ECHO command expects only a single argument.
    ///
    /// # Errors
    /// The validation errors are returned as SimpleError RESP type.
    fn execute(&self, args: &[&RespType]) -> (RespType, Option<BytesMut>) {
        if args.len() == 0 {
            return (
                RespType::SimpleError("ERR wrong number of arguments for command".into()),
                None,
            );
        }

        (args[0].clone(), None)
    }
}

impl<'a> CommandHandler for Echo<'a> {
    /// Execute the ECHO command, and then write the output to the response TCP stream.
    async fn handle(&mut self, args: &[&RespType]) -> anyhow::Result<usize> {
        let (res, _) = self.execute(args);
        RespType::write_to_stream(self.stream, &res).await
    }
}
