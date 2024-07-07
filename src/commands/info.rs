use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::SimpleError;

use anyhow::{anyhow, Result};

const ALL_INFO_ARGS: [InfoArg; 1] = [InfoArg::REPLICATION];

/// Struct for the INFO command.
pub struct Info {}

impl CommandExecutor for Info {
    /// Returns the server info in BulkString format.
    /// Specific sections of info can be selected by specifying optional parameters.
    /// If no optional parameters are specified, all sections are returned.
    ///
    /// # Supported optional params
    /// - replication : Master/replica replication information.
    fn execute(&mut self, args: &[&RespType]) -> RespType {
        let mut info_args = vec![];

        // get the info sections to be returned
        if args.len() == 0 {
            info_args.extend(ALL_INFO_ARGS);
        } else {
            for arg in args {
                let info_arg = InfoArg::parse(*arg);
                match info_arg {
                    Ok(ia) => {
                        info_args.push(ia);
                    }
                    Err(e) => return SimpleError(format!("{}", e)),
                }
            }
        }

        // append section infos in a loop
        let mut info = String::new();

        for info_arg in info_args {
            let section = match info_arg {
                InfoArg::REPLICATION => "# Replication\nrole:master\n",
            };

            info.push_str(section)
        }

        return RespType::BulkString(info);
    }
}

/// Arguments supported by the INFO command.
enum InfoArg {
    /// Info about replication.
    REPLICATION,
}

impl InfoArg {
    fn parse(arg: &RespType) -> Result<InfoArg> {
        let s = match arg {
            RespType::BulkString(s) => s,
            _ => return Err(anyhow!("ERR Invalid argument. Value must be a bulk string")),
        };

        match s.to_uppercase().as_str() {
            "REPLICATION" => Ok(InfoArg::REPLICATION),
            _ => Err(anyhow!("Invalid argument for INFO command")),
        }
    }
}
