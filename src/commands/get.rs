use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::{BulkString, SimpleError};
use crate::storage::store::Store;

/// Struct for the GET command.
/// It holds the pointer to the backing data store.
pub struct Get<'a> {
    /// Pointer to the data store.
    store: &'a Store,
}

impl<'a> Get<'a> {
    /// Creates a new GET command struct.
    pub fn new(store: &Store) -> Get {
        Get { store }
    }
}

impl<'a> CommandExecutor for Get<'a> {
    /// Gets the value stored against the key from the data store
    /// The key is provided as part of the RESP arguments.
    ///
    /// The value stored against the key will be returned as a BulkString. If there's no
    /// value associated with the key, a NullBulkString is returned instead.
    ///
    /// # Validations
    /// GET command expects only a single argument, which is the key (in BulkString format).
    ///
    /// # Errors
    /// The validation errors are returned as SimpleError RESP type.
    fn execute(&mut self, args: &[&RespType]) -> RespType {
        if args.len() != 1 {
            return SimpleError("ERR wrong number of arguments for command".into());
        }

        let key = args[0];
        let key = match key {
            BulkString(k) => k,
            _ => return SimpleError("ERR Invalid argument. Key must be a bulk string".into()),
        };

        let val = self.store.get(key);
        if val.is_none() {
            return RespType::null_bulk_string();
        }

        BulkString(val.unwrap().val().to_string())
    }
}
