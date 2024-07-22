use crate::commands::traits::{CommandBuilder, CommandExecutor, CommandHandler};
use crate::protocol::resp::types::RespType;
use crate::protocol::resp::types::RespType::{BulkString, SimpleError, SimpleString};
use crate::server::info::{Role, ServerConfig};
use anyhow::anyhow;
use bytes::BytesMut;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// Struct for the PSYNC command.
pub struct Psync<'a> {
    stream: &'a mut TcpStream,
    /// Used to fetch server info
    server_config: &'a ServerConfig,
}

impl<'a> Psync<'a> {
    pub fn new(stream: &'a mut TcpStream, server_config: &'a ServerConfig) -> Psync<'a> {
        Psync {
            stream,
            server_config,
        }
    }

    fn is_unknown_replication_id(repl_id: &str) -> bool {
        repl_id == "?"
    }

    fn is_null_offset(offset: &str) -> bool {
        offset == "-1"
    }
}

impl<'a> CommandExecutor for Psync<'a> {
    /// Blindly return a FULLRESYNC response for now.
    /// Supports only first-time replica connection as of now.
    /// TODO: Actual replication configuration
    fn execute(&self, args: &[&RespType]) -> (RespType, Option<BytesMut>) {
        match &self.server_config.role {
            Role::MASTER => {}
            Role::SLAVE => {
                return (
                    SimpleError("PSYNC cannot be performed by a slave server".into()),
                    None,
                )
            }
        };

        if args.len() < 2 {
            return (
                SimpleError("ERR insufficient arguments for command".into()),
                None,
            );
        }

        // parse replication_id
        let replication_id = args[0];
        let replication_id = match replication_id {
            BulkString(k) => k,
            _ => {
                return (
                    SimpleError(
                        "ERR Invalid argument. replication_id must be a bulk string".into(),
                    ),
                    None,
                )
            }
        };

        // parse offset
        let offset = args[1];
        let offset = match offset {
            BulkString(v) => v,
            _ => {
                return (
                    SimpleError("ERR Invalid argument. offset must be a bulk string".into()),
                    None,
                )
            }
        };

        // when slave is connecting first time replication_id should be `?` and offset should be `-1`
        if Self::is_unknown_replication_id(replication_id.as_str())
            && Self::is_null_offset(offset.as_str())
        {
            let empty_file_payload = hex::decode("524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2").unwrap();
            let byte_data_prefix = format!("${}\r\n", empty_file_payload.len());
            let mut payload_bytes = BytesMut::from(byte_data_prefix.as_bytes());
            payload_bytes.extend_from_slice(empty_file_payload.as_slice());
            return (
                SimpleString(format!(
                    "FULLRESYNC {} 0",
                    self.server_config.replication.id
                )),
                Some(payload_bytes),
            );
        }

        return (
            SimpleError("ERR - Supports FULLRESYNC for first-time replica connection only".into()),
            None,
        );
    }
}

impl<'a> CommandHandler for Psync<'a> {
    /// Execute the PSYNC command, and then write the output to the response TCP stream.
    /// The PSYNC output can have the RDB file as its output. That will also be written to
    /// the response stream.
    async fn handle(&mut self, args: &[&RespType]) -> anyhow::Result<usize> {
        let (res, payload_bytes) = self.execute(args);
        let resp_bytes = RespType::write_to_stream(self.stream, &res).await;
        let resp_bytes = match resp_bytes {
            Ok(b) => b,
            Err(_) => return Err(anyhow!("Failed to write data into response stream")),
        };

        match payload_bytes {
            None => Ok(resp_bytes),
            Some(b) => {
                let raw_bytes = self.stream.write(b.as_ref()).await;
                match raw_bytes {
                    Ok(bytes_written) => Ok(resp_bytes + bytes_written),
                    Err(e) => Err(anyhow!("Failed to write RDB payload bytes: {}", e)),
                }
            }
        }
    }
}

impl<'a> CommandBuilder for Psync<'a> {
    /// Returns a PSYNC command in RESP array format.
    fn build(args: Option<&[&RespType]>) -> RespType {
        let mut cmd = vec![RespType::BulkString("PSYNC".to_string())];
        if args.is_some() {
            let cmd_args = args
                .unwrap()
                .iter()
                .map(|&r| r.clone())
                .collect::<Vec<RespType>>();
            cmd.extend(cmd_args);
        }

        RespType::Array(cmd)
    }
}
