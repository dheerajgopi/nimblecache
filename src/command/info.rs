use crate::{replication::Replication, resp::types::RespType};

use super::CommandError;

const ALL_INFO_ARGS: [InfoArg; 1] = [InfoArg::REPLICATION];

/// Represents the INFO command in Nimblecache.
#[derive(Debug, Clone)]
pub struct Info {
    args: Vec<InfoArg>,
}

impl Info {
    /// Creates a new `Info` instance from the given arguments.
    ///
    /// # Returns
    ///
    /// * `Ok(Info)` if parsing succeeds.
    /// * `Err(CommandError)` if parsing fails.
    pub fn with_args(args: Vec<RespType>) -> Result<Info, CommandError> {
        let mut info_args = vec![];

        // get the info sections to be returned
        if args.len() == 0 {
            info_args.extend(ALL_INFO_ARGS);
        } else {
            for arg in args.iter() {
                let info_arg = InfoArg::parse(arg);
                match info_arg {
                    Ok(ia) => {
                        info_args.push(ia);
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(Info { args: info_args })
    }

    /// Executes the INFO command.
    ///
    /// # Returns
    ///
    /// Returns a `BulkString` with server info.
    pub fn apply(&self, replication: &Replication) -> RespType {
        // append section infos in a loop
        let mut info = String::new();

        for info_arg in self.args.iter() {
            let section = match info_arg {
                InfoArg::REPLICATION => {
                    format!("# Replication\n{}\n", replication.info_str())
                }
            };

            info.push_str(section.as_str())
        }
        RespType::BulkString(info)
    }
}

#[derive(Debug, Clone)]
/// Arguments supported by the INFO command.
enum InfoArg {
    /// Info about replication.
    REPLICATION,
}

impl InfoArg {
    /// Parse the optional params for valid values.
    ///
    /// # Validations
    /// - Optional params should be in BulkString format.
    /// - Valid optional param values - `REPLICATION`.
    fn parse(arg: &RespType) -> Result<InfoArg, CommandError> {
        let s = match arg {
            RespType::BulkString(s) => s,
            _ => {
                return Err(CommandError::Other(String::from(
                    "Invalid argument. INFO parameters must be in bulk string format",
                )))
            }
        };

        match s.to_lowercase().as_str() {
            "replication" => Ok(InfoArg::REPLICATION),
            _ => Err(CommandError::Other(String::from(
                "Invalid argument for INFO command",
            ))),
        }
    }
}
