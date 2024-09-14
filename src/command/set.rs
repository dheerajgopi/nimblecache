use time::{Duration, OffsetDateTime};

use crate::{
    resp::types::RespType,
    storage::db::{Value, DB},
};

use super::CommandError;

/// Represents the SET command in Nimblecache.
#[derive(Debug, Clone)]
pub struct Set {
    key: String,
    value: String,
    expiry: Option<OffsetDateTime>,
}

impl Set {
    /// Creates a new `Set` instance from the given arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `RespType` representing the arguments to the SET command.
    ///
    /// # Returns
    ///
    /// * `Ok(Set)` if parsing succeeds.
    /// * `Err(CommandError)` if parsing fails.
    pub fn with_args(args: Vec<RespType>) -> Result<Set, CommandError> {
        if args.len() < 2 {
            return Err(CommandError::Other(String::from(
                "Wrong number of arguments specified for 'SET' command",
            )));
        }

        // parse key
        let key = &args[0];
        let key = match key {
            RespType::BulkString(k) => k,
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. Key must be a bulk string",
                )));
            }
        };

        // parse value
        let value = &args[1];
        let value = match value {
            RespType::BulkString(v) => v.to_string(),
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. Value must be a bulk string",
                )));
            }
        };

        let mut expiry: Option<OffsetDateTime> = None;

        // parse the options if provided
        if args.len() > 2 {
            let option_args: Vec<&RespType> = args[2..].iter().collect();
            let mut start_idx: usize = 0;

            // read till end of the argument list to parse all options.
            while start_idx < option_args.len() {
                let (opt, nxt_idx) = match SetOption::parse(&option_args, start_idx) {
                    Ok((o, nxt_idx)) => (o, nxt_idx),
                    Err(e) => return Err(CommandError::Other(format!("{}", e))),
                };

                // set expiry
                let now = OffsetDateTime::now_utc();
                match opt {
                    SetOption::PX(ttl) => {
                        expiry = Some(now.saturating_add(Duration::milliseconds(ttl as i64)));
                    }
                    SetOption::PXAT(exp_ts_utc) => {
                        expiry = Some(
                            OffsetDateTime::UNIX_EPOCH
                                .saturating_add(Duration::milliseconds(exp_ts_utc as i64)),
                        );
                    }
                };

                start_idx = nxt_idx;
            }
        }

        Ok(Set {
            key: key.to_string(),
            value,
            expiry,
        })
    }

    /// Executes the SET command.
    ///
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// # Returns
    ///
    /// It returns an 'OK` as a `BulkString` if value is successfully written.
    pub fn apply(&self, db: &DB) -> RespType {
        match db.set(
            self.key.clone(),
            Value::String(self.value.clone()),
            self.expiry,
        ) {
            Ok(_) => RespType::BulkString("OK".to_string()),
            Err(e) => RespType::SimpleError(format!("{}", e)),
        }
    }

    pub fn build_command(&self) -> RespType {
        let mut cmd = vec![
            RespType::BulkString(String::from("SET")),
            RespType::BulkString(self.key.clone()),
            RespType::BulkString(self.value.clone()),
        ];

        if let Some(exp_ts) = self.expiry {
            let ms_from_epoch = (exp_ts - OffsetDateTime::UNIX_EPOCH).whole_milliseconds() as u64;
            cmd.push(RespType::BulkString(String::from("PXAT")));
            cmd.push(RespType::BulkString(ms_from_epoch.to_string()));
        }

        RespType::Array(cmd)
    }
}

/// Options supported by the SET command.
enum SetOption {
    /// TTL for the key specified in milliseconds.
    PX(u64),
    /// Specified unix-time for the key expiry specified in milliseconds.
    PXAT(u64),
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
    pub fn parse(opts: &[&RespType], start_idx: usize) -> Result<(SetOption, usize), CommandError> {
        if start_idx >= opts.len() {
            return Err(CommandError::Other(String::from("Invalid arguments")));
        }

        let opt_name = opts[start_idx];
        let opt_name = match opt_name {
            RespType::BulkString(o) => o,
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. All arguments should be in bulk string format",
                )));
            }
        };

        match opt_name.to_lowercase().as_str() {
            "px" => Self::get_px(opts, start_idx),
            "pxat" => Self::get_pxat(opts, start_idx),
            _ => Err(CommandError::Other(String::from(
                "Invalid option specified",
            ))),
        }
    }

    /// Parse and return the value for PX option along with the next index to start the parsing from the argument list.
    fn get_px(opts: &[&RespType], start_idx: usize) -> Result<(SetOption, usize), CommandError> {
        let px_val_idx = start_idx + 1;
        if px_val_idx >= opts.len() {
            return Err(CommandError::Other(String::from(
                "Value for PX is not specified. Provide an integer value",
            )));
        }

        let px_val = opts[px_val_idx];
        let px_val = match px_val {
            RespType::BulkString(p) => p,
            _ => {
                return Err(CommandError::Other(String::from(
                    "Value for PX should be in bulk string format",
                )));
            }
        };
        let px_val = px_val.parse::<u64>();
        let px_val = match px_val {
            Ok(v) => v,
            Err(_) => {
                return Err(CommandError::Other(String::from(
                    "Value for PX should be an integer",
                )));
            }
        };

        Ok((SetOption::PX(px_val), px_val_idx + 1))
    }

    /// Parse and return the value for PXAT option along with the next index to start the parsing from the argument list.
    fn get_pxat(opts: &[&RespType], start_idx: usize) -> Result<(SetOption, usize), CommandError> {
        let pxat_val_idx = start_idx + 1;
        if pxat_val_idx >= opts.len() {
            return Err(CommandError::Other(String::from(
                "Value for PXAT is not specified. Provide an integer value",
            )));
        }

        let pxat_val = opts[pxat_val_idx];
        let pxat_val = match pxat_val {
            RespType::BulkString(p) => p,
            _ => {
                return Err(CommandError::Other(String::from(
                    "Value for PXAT should be in bulk string format",
                )));
            }
        };
        let pxat_val = pxat_val.parse::<u64>();
        let pxat_val = match pxat_val {
            Ok(v) => v,
            Err(_) => {
                return Err(CommandError::Other(String::from(
                    "Value for PXAT should be an integer",
                )));
            }
        };

        Ok((SetOption::PXAT(pxat_val), pxat_val_idx + 1))
    }
}
