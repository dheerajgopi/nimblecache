use crate::commands::traits::{CommandExecutor, CommandHandler};
use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::{BulkString, SimpleError};
use crate::storage::store::Store;
use anyhow::Result;
use bytes::BytesMut;
use tokio::net::TcpStream;

/// Struct for the GET command.
/// It holds the pointer to the backing data store.
pub struct Get<'a> {
    stream: &'a mut TcpStream,
    /// Pointer to the data store.
    store: &'a Store,
}

impl<'a> Get<'a> {
    /// Creates a new GET command struct.
    pub fn new(stream: &'a mut TcpStream, store: &'a Store) -> Get<'a> {
        Get { stream, store }
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
    fn execute(&self, args: &[&RespType]) -> (RespType, Option<BytesMut>) {
        if args.len() != 1 {
            return (
                SimpleError("ERR wrong number of arguments for command".into()),
                None,
            );
        }

        let key = args[0];
        let key = match key {
            BulkString(k) => k,
            _ => {
                return (
                    SimpleError("ERR Invalid argument. Key must be a bulk string".into()),
                    None,
                )
            }
        };

        let value = self.store.get(key);
        if value.is_none() {
            return (RespType::null_bulk_string(), None);
        }

        let value = value.unwrap();
        if value.has_expired() {
            return (RespType::null_bulk_string(), None);
        }

        (BulkString(value.val().to_string()), None)
    }
}

impl<'a> CommandHandler for Get<'a> {
    /// Execute the GET command, and then write the output to the response TCP stream.
    async fn handle(&mut self, args: &[&RespType]) -> Result<usize> {
        let (res, _) = self.execute(args);
        RespType::write_to_stream(self.stream, &res).await
    }
}
