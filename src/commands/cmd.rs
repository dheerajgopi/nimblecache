use crate::commands::traits::CommandExecutor;
use crate::commands::{echo, get, info, ping, psync, replconf, set};
use crate::protocol::resp::types::RespType;
use crate::server::info::ServerConfig;
use crate::storage::store::Store;
use anyhow::{anyhow, Result};
use bytes::BytesMut;

/// Unit struct used for executing various Nimblecache commands.
pub struct Cmd {}

impl Cmd {
    /// Extract the command and its arguments from the given RESP value, and execute the same.
    ///
    /// # Validations
    /// A command should always be in Array (of BulkStrings) RESP format. The first item should be the command name,
    /// and the rest of the items will be the arguments to the command.
    ///
    /// # Errors
    /// The validation errors are returned as SimpleError RESP type.
    pub fn execute(
        resp_val: &RespType,
        store: &Store,
        server_config: &ServerConfig,
    ) -> CmdResponse {
        let cmd_name_and_args = Cmd::extract_command_name_and_args(resp_val);
        let (cmd_name, args) = match cmd_name_and_args {
            Ok(cmd) => (cmd.0, cmd.1),
            Err(e) => {
                return CmdResponse::FAILURE(FailureResponse {
                    error: RespType::SimpleError(format!("(error) {:?}", e)),
                })
            }
        };
        let args = args.iter().map(|a| a).collect::<Vec<&RespType>>();
        let args = args.as_slice();

        let (res, bytes_payload) = match cmd_name.to_uppercase().as_str() {
            "ECHO" => echo::Echo {}.execute(args),
            "GET" => get::Get::new(store).execute(args),
            "INFO" => info::Info::new(server_config).execute(args),
            "PING" => ping::Ping {}.execute(args),
            "SET" => set::Set::new(store).execute(args),
            "REPLCONF" => replconf::Replconf {}.execute(args),
            "PSYNC" => psync::Psync::new(server_config).execute(args),
            _ => {
                return CmdResponse::FAILURE(FailureResponse {
                    error: RespType::SimpleError(format!(
                        "(error) unknown command '{:?}'",
                        cmd_name
                    )),
                })
            }
        };

        CmdResponse::SUCCESS(SuccessResponse {
            output: res,
            bytes_payload,
            cmd_name,
        })
    }

    fn extract_command_name_and_args(resp_val: &RespType) -> Result<(String, Vec<RespType>)> {
        let resp_arr = match resp_val {
            RespType::Array(arr) => arr,
            _ => return Err(anyhow!("Invalid command format")),
        };

        if resp_arr.len() == 0 {
            return Err(anyhow!("No commands are provided"));
        }

        let cmd_name = resp_arr.first().unwrap();
        let cmd_name = match cmd_name {
            RespType::BulkString(name) => name,
            _ => return Err(anyhow!("Invalid command format")),
        };

        let args = resp_arr
            .into_iter()
            .skip(1)
            .map(|arg| arg.clone())
            .collect::<Vec<RespType>>();

        Ok((cmd_name.into(), args))
    }
}

/// Wraps the success/failure response returned after executing the Nimblecache commands
pub enum CmdResponse {
    SUCCESS(SuccessResponse),
    FAILURE(FailureResponse),
}

impl CmdResponse {
    /// Returns the RESP response and the optional bytes payload (for e.g. PSYNC rdb files)
    pub fn resp_output(self) -> (RespType, Option<BytesMut>) {
        match self {
            CmdResponse::SUCCESS(s) => (s.output, s.bytes_payload),
            CmdResponse::FAILURE(f) => (f.error, None),
        }
    }

    /// Successful handshake happens when PSYNC command from replica returns SUCCESS response.
    pub fn is_successful_handshake_from_replica(&self) -> bool {
        match self {
            CmdResponse::SUCCESS(s) => {
                if s.cmd_name == "PSYNC" {
                    true
                } else {
                    false
                }
            }
            CmdResponse::FAILURE(_) => false,
        }
    }
}

pub struct SuccessResponse {
    output: RespType,
    bytes_payload: Option<BytesMut>,
    cmd_name: String,
}

pub struct FailureResponse {
    error: RespType,
}
