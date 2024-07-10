use crate::commands::traits::{CommandBuilder, CommandExecutor};
use crate::protocol::resp::types::RespType;

/// Struct for the REPLCONF command.
pub struct Replconf {}

impl CommandExecutor for Replconf {
    /// Returns an OK for now.
    /// TODO: Actual replication configuration
    fn execute(&mut self, _: &[&RespType]) -> RespType {
        return RespType::SimpleString("OK".into());
    }
}

impl CommandBuilder for Replconf {
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
