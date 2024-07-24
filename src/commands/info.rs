use std::sync::Arc;

use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::SimpleError;
use crate::{commands::traits::CommandExecutor, server::info::ServerConfig};

use crate::commands::traits::CommandHandler;
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::net::TcpStream;

const ALL_INFO_ARGS: [InfoArg; 1] = [InfoArg::REPLICATION];

/// Struct for the INFO command.
pub struct Info<'a> {
    stream: &'a mut TcpStream,
    /// Used to fetch server info
    server_config: Arc<ServerConfig>,
}

impl<'a> Info<'a> {
    /// Create new Info command struct
    pub fn new(stream: &'a mut TcpStream, server_config: Arc<ServerConfig>) -> Info<'a> {
        Info {
            stream,
            server_config,
        }
    }
}

impl<'a> CommandExecutor for Info<'a> {
    /// Returns the server info in BulkString format.
    /// Specific sections of info can be selected by specifying optional parameters.
    /// If no optional parameters are specified, all sections are returned.
    ///
    /// # Supported optional params
    /// - replication : Master/replica replication information.
    fn execute(&self, args: &[&RespType]) -> (RespType, Option<BytesMut>) {
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

        let server_config = self.server_config.as_ref();

        for info_arg in info_args {
            let section = match info_arg {
                InfoArg::REPLICATION => {
                    format!("# Replication\n{}\n", server_config.info_replication())
                }
            };

            info.push_str(section.as_str())
        }

        return (RespType::BulkString(info), None);
    }
}

impl<'a> CommandHandler for Info<'a> {
    /// Execute the INFO command, and then write the output to the response TCP stream.
    async fn handle(&mut self, args: &[&RespType]) -> Result<usize> {
        let (res, _) = self.execute(args);
        RespType::write_to_stream(self.stream, &res).await
    }
}

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
