use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::{SimpleError, BulkString};
use crate::storage::store::Store;

pub struct Get<'a> {
    store: &'a Store
}

impl<'a> Get<'a> {
    pub fn new(store: &Store) -> Get {
        Get {
            store
        }
    }
}

impl<'a> CommandExecutor for Get<'a> {
    fn execute(&mut self, args: &[&RespType]) -> RespType {
        if args.len() != 1 {
            return SimpleError("ERR wrong number of arguments for command".into());
        }

        let key = args[0];
        let key = match key {
            BulkString(k) => {
                k
            }
            _ => {
                return SimpleError("ERR Invalid argument. Key must be a bulk string".into())
            }
        };

        let val = self.store.get(key);
        if val.is_none() {
            return RespType::null_bulk_string()
        }

        BulkString(val.unwrap())
    }
}