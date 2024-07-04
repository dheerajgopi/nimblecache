use crate::protocol::resp::types::RespType;

/// Trait for executing Nimblecache commands.
pub trait CommandExecutor {
    /// Execute the Nimblecache command based on the input arguments provided as a RESP type.
    /// The return value should also be of RESP type.
    fn execute(&mut self, args: &[&RespType]) -> RespType;
}
