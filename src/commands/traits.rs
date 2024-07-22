use crate::protocol::resp::types::RespType;
use bytes::BytesMut;

/// Trait for executing Nimblecache commands.
pub trait CommandExecutor {
    /// Execute the Nimblecache command based on the input arguments provided as a RESP type.
    /// The return value should be of RESP type and an optional byte array.
    /// The RESP type is the response to the command.
    /// The byte array will be None most of the time. It will have some value only in cases such as
    /// PSYNC command where server has to send certain byte data just after the RESP response.
    fn execute(&self, args: &[&RespType]) -> (RespType, Option<BytesMut>);
}

/// Trait for building Nimblecache commands which is essentially a RESP array
/// of RESP bulk strings
pub trait CommandBuilder {
    /// Return the Nimblecache command as a RESP array.
    /// The first item is the command itself.
    /// Rest of the items in the array are the arguments supplied to the command.
    fn build(args: Option<&[&RespType]>) -> RespType;
}

/// Trait for handling Nimblecache commands. "Handling" a command means that the Nimblecache
/// command will be executed and the output will be written to the response stream.
pub trait CommandHandler: CommandExecutor {
    /// Execute the Nimblecache command based on the input arguments provided as a RESP type.
    /// The output of the command is then written to the response stream.
    /// The return value is the number of bytes written to the response stream.
    async fn handle(&mut self, args: &[&RespType]) -> anyhow::Result<usize>;
}
