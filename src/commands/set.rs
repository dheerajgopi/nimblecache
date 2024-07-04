use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::{BulkString, SimpleError};
use crate::storage::store::Store;

pub struct Set<'a> {
    store: &'a Store
}

impl<'a> Set<'a> {
    pub fn new(store: &Store) -> Set {
        Set {
            store
        }
    }
}

impl<'a> CommandExecutor for Set<'a> {
    fn execute(&mut self, args: &[&RespType]) -> RespType {
        if args.len() != 2 {
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

        let val = args[1];
        let val = match val {
            BulkString(v) => {
                v
            }
            _ => {
                return SimpleError("ERR Invalid argument. Value must be a bulk string".into())
            }
        };

        self.store.put(key.clone(), val.clone());

        BulkString("OK".into())
    }
}