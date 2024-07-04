use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::{BulkString, SimpleError};
use crate::storage::store::Store;

/// Struct for the SET command.
/// It holds the pointer to the backing data store.
pub struct Set<'a> {
    /// Pointer to the data store.
    store: &'a Store,
}

impl<'a> Set<'a> {
    /// Creates a new SET command struct.
    pub fn new(store: &Store) -> Set {
        Set { store }
    }
}

impl<'a> CommandExecutor for Set<'a> {
    /// Sets a value against the key in the data store. If a value was already attached to the key in
    /// the data store, it will be updated with the new value. Both key and value are provided as
    /// part of the RESP arguments.
    ///
    /// If the value is set against a key successfully, an "OK" will be returned in BulkString format.
    ///
    /// # Validations
    /// SET command expects two arguments in the below order
    /// - key (BulkString)
    /// - value (BulkString)
    ///
    /// # Errors
    /// The validation errors are returned as SimpleError RESP type.
    fn execute(&mut self, args: &[&RespType]) -> RespType {
        if args.len() != 2 {
            return SimpleError("ERR wrong number of arguments for command".into());
        }

        let key = args[0];
        let key = match key {
            BulkString(k) => k,
            _ => return SimpleError("ERR Invalid argument. Key must be a bulk string".into()),
        };

        let val = args[1];
        let val = match val {
            BulkString(v) => v,
            _ => return SimpleError("ERR Invalid argument. Value must be a bulk string".into()),
        };

        self.store.put(key.clone(), val.clone());

        BulkString("OK".into())
    }
}
