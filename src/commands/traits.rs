use crate::protocol::resp::types::RespType;

/// Trait for executing Nimblecache commands.
pub trait CommandExecutor {
    /// Execute the Nimblecache command based on the input arguments provided as a RESP type.
    /// The return value should also be of RESP type.
    fn execute(&mut self, args: &[&RespType]) -> RespType;
}

/// Trait for building Nimblecache commands which is essentially a RESP array
/// of RESP bulk strings
pub trait CommandBuilder {
    /// Return the Nimblecache command as a RESP array.
    /// The first item is the command itself.
    /// Rest of the items in the array are the arguments supplied to the command.
    fn build(args: Option<&[&RespType]>) -> RespType;
}
