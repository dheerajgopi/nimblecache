use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::{BulkString, SimpleError};
use crate::storage::store::Store;
use crate::storage::value::StringValue;
use anyhow::{anyhow, Result};

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
    /// # Supported options
    /// - PX _ttl_in_milliseconds_ : Set the specified TTL in milliseconds (a positive integer).
    ///
    /// # Validations
    /// SET command expects two arguments in the below order
    /// - key (BulkString)
    /// - value (BulkString)
    ///
    /// # Errors
    /// The validation errors are returned as SimpleError RESP type.
    fn execute(&mut self, args: &[&RespType]) -> RespType {
        if args.len() < 2 {
            return SimpleError("ERR insufficient arguments for command".into());
        }

        // parse key
        let key = args[0];
        let key = match key {
            BulkString(k) => k,
            _ => return SimpleError("ERR Invalid argument. Key must be a bulk string".into()),
        };

        // parse value
        let val = args[1];
        let val = match val {
            BulkString(v) => v,
            _ => return SimpleError("ERR Invalid argument. Value must be a bulk string".into()),
        };
        let mut val = StringValue::new(val.clone(), None);

        // parse the options if provided
        if args.len() > 2 {
            let option_args: Vec<&RespType> = args[2..].into();
            let mut start_idx: usize = 0;

            // read till end of the argument list to parse all options.
            while start_idx < option_args.len() {
                let (opt, nxt_idx) = match SetOption::parse(&option_args, start_idx) {
                    Ok((o, nxt_idx)) => (o, nxt_idx),
                    Err(e) => return SimpleError(format!("{}", e)),
                };

                // set expiry
                match opt {
                    SetOption::PX(ttl) => {
                        val.set_ttl(ttl);
                    }
                };

                start_idx = nxt_idx;
            }
        }

        self.store.put(key.clone(), val);

        BulkString("OK".into())
    }
}

/// Options supported by the SET command.
enum SetOption {
    /// Expiry time for the key specified in milliseconds.
    PX(u128),
}

impl SetOption {
    /// Parse the argument list from the specified start index and return the first option
    /// along with the next index to start the parsing from.
    ///
    /// # Errors
    /// Errors are returned if:
    /// - Invalid option name is specified.
    /// - Wrong data type for any argument.
    /// - Slice index errors.
    pub fn parse(opts: &[&RespType], start_idx: usize) -> Result<(SetOption, usize)> {
        if start_idx >= opts.len() {
            return Err(anyhow!("Invalid arguments"));
        }

        let opt_name = opts[start_idx];
        let opt_name = match opt_name {
            BulkString(o) => o,
            _ => {
                return Err(anyhow!(
                    "Invalid argument. All arguments should be in bulk string format"
                ));
            }
        };

        match opt_name.to_uppercase().as_str() {
            "PX" => Self::get_px(opts, start_idx),
            _ => {
                return Err(anyhow!("Invalid option specified"));
            }
        }
    }

    /// Parse and return the value for PX option along with the next index to start the parsing from the argument list.
    fn get_px(opts: &[&RespType], start_idx: usize) -> Result<(SetOption, usize)> {
        let px_val_idx = start_idx + 1;
        if px_val_idx >= opts.len() {
            return Err(anyhow!(
                "Value for PX is not specified. Provide an integer value"
            ));
        }

        let px_val = opts[px_val_idx];
        let px_val = match px_val {
            BulkString(p) => p,
            _ => {
                return Err(anyhow!("Value for PX should be in bulk string format"));
            }
        };
        let px_val = px_val.parse::<u128>();
        let px_val = match px_val {
            Ok(v) => v,
            Err(_) => {
                return Err(anyhow!("Value for PX should be an integer"));
            }
        };

        Ok((SetOption::PX(px_val), px_val_idx + 1))
    }
}
