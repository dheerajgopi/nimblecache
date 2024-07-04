use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;

pub struct Info {}

impl CommandExecutor for Info {
    fn execute(&mut self, _: &[&RespType]) -> RespType {
        return RespType::BulkString("# Server\n\nnimblecache_version: 0.1.0".into());
    }
}
