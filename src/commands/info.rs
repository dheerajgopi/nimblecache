use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;

/// Struct for the INFO command.
pub struct Info {}

impl CommandExecutor for Info {
    /// Returns the server info in BulkString format.
    fn execute(&mut self, _: &[&RespType]) -> RespType {
        return RespType::BulkString("# Server\n\nnimblecache_version: 0.1.0".into());
    }
}
