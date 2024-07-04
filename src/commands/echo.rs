use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;

/// Struct for the ECHO command.
pub struct Echo {}

impl CommandExecutor for Echo {
    /// Returns the given argument as it is. In other words, it simply echoes back the given argument.
    ///
    /// # Validations
    /// ECHO command expects only a single argument.
    ///
    /// # Errors
    /// The validation errors are returned as SimpleError RESP type.
    fn execute(&mut self, args: &[&RespType]) -> RespType {
        if args.len() == 0 {
            return RespType::SimpleError("ERR wrong number of arguments for command".into());
        }

        args[0].clone()
    }
}
