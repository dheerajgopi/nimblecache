use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::SimpleError;
use crate::{commands::traits::CommandExecutor, server::info::ServerConfig};

use anyhow::{anyhow, Result};
use bytes::BytesMut;

const ALL_INFO_ARGS: [InfoArg; 1] = [InfoArg::REPLICATION];

/// Struct for the INFO command.
pub struct Info<'a> {
    /// Used to fetch server info
    server_config: &'a ServerConfig,
}

impl<'a> Info<'a> {
    /// Create new Info command struct
    pub fn new(server_config: &ServerConfig) -> Info {
        Info { server_config }
    }
}

impl<'a> CommandExecutor for Info<'a> {
    /// Returns the server info in BulkString format.
    /// Specific sections of info can be selected by specifying optional parameters.
    /// If no optional parameters are specified, all sections are returned.
    ///
    /// # Supported optional params
    /// - replication : Master/replica replication information.
    fn execute(&mut self, args: &[&RespType]) -> (RespType, Option<BytesMut>) {
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
                    Err(e) => return (SimpleError(format!("{}", e)), None),
                }
            }
        }

        // append section infos in a loop
        let mut info = String::new();

        for info_arg in info_args {
            let section = match info_arg {
                InfoArg::REPLICATION => {
                    format!("# Replication\n{}\n", self.server_config.info_replication())
                }
            };

            info.push_str(section.as_str())
        }

        return (RespType::BulkString(info), None);
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
