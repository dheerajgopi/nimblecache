use crate::commands::echo::Echo;
use crate::commands::get::Get;
use crate::commands::info::Info;
use crate::commands::ping::Ping;
use crate::commands::psync::Psync;
use crate::commands::replconf::Replconf;
use crate::commands::set::Set;
use crate::commands::traits::CommandHandler;
use crate::protocol::resp::types::RespType;
use crate::server::info::ServerConfig;
use crate::storage::store::Store;
use anyhow::{anyhow, Error, Result};
use bytes::BytesMut;
use std::sync::Arc;
use tokio::net::TcpStream;

const EMPTY_ARGS: Vec<RespType> = vec![];

/// RespCommandHandler can read RESP values from a TcpStream, execute it, and write the output RESP values into the same TcpStream.
pub enum RespCommandHandler<'a> {
    ECHO(Echo<'a>),
    GET(Get<'a>),
    INFO(Info<'a>),
    PING(Ping<'a>),
    PSYNC(Psync<'a>),
    REPLCONF(Replconf<'a>),
    SET(Set<'a>),
    // Special case for handling requests with 0 bytes
    NULL,
    // Special handler for error responses
    ERR(ErrHandler<'a>),
}

impl<'a> RespCommandHandler<'a> {
    /// Creates a new command handler using the given TcpStream.
    ///
    /// It will extract the command, and its arguments from the RESP values in the request stream.
    /// Based on the command, it will return the relevant handler.
    ///
    /// The return value is a tuple with the command handler as first item, and the arguments as
    /// the second item.
    ///
    /// Special case:
    /// If the number of bytes read is 0, a NULL handler is returned.
    ///
    /// # Errors
    /// - If request parsing fails.
    /// - If the command is invalid.
    pub async fn from(
        stream: &'a mut TcpStream,
        buffer: BytesMut,
        store: &'a Store,
        server_config: Arc<ServerConfig>,
    ) -> Result<(RespCommandHandler<'a>, Vec<RespType>)> {
        // parse RESP value from the stream
        let mut buf = buffer.clone();
        let (resp_val, _) = match RespType::from_stream(stream, &mut buf).await {
            Ok((cmd, b)) => {
                if cmd.is_none() {
                    return Ok((RespCommandHandler::NULL, EMPTY_ARGS));
                }

                (cmd.unwrap(), b)
            }
            Err(e) => {
                return Err(anyhow!("Bad request: {}", e));
            }
        };

        // extract the command and its arguments
        let cmd_name_and_args = Self::extract_command_name_and_args(&resp_val);
        let (cmd_name, args) = match cmd_name_and_args {
            Ok(cmd) => (cmd.0, cmd.1),
            Err(e) => return Err(anyhow!("Bad request: {}", e)),
        };

        // return appropriate command handler
        let handler = match cmd_name.to_uppercase().as_str() {
            "ECHO" => RespCommandHandler::ECHO(Echo::new(stream)),
            "GET" => RespCommandHandler::GET(Get::new(stream, store)),
            "INFO" => RespCommandHandler::INFO(Info::new(stream, server_config)),
            "PING" => RespCommandHandler::PING(Ping::new(stream)),
            "SET" => RespCommandHandler::SET(Set::new(stream, store)),
            "REPLCONF" => RespCommandHandler::REPLCONF(Replconf::new(stream)),
            "PSYNC" => RespCommandHandler::PSYNC(Psync::new(stream, Arc::clone(&server_config))),
            _ => return Err(anyhow!("Unknown command: {}", cmd_name)),
        };

        Ok((handler, args))
    }

    /// Execute the command, and write the output into the response stream.
    pub async fn handle(&mut self, args: &[&RespType]) -> Result<usize> {
        match self {
            RespCommandHandler::ECHO(echo) => echo.handle(args).await,
            RespCommandHandler::GET(get) => get.handle(args).await,
            RespCommandHandler::INFO(info) => info.handle(args).await,
            RespCommandHandler::PING(ping) => ping.handle(args).await,
            RespCommandHandler::PSYNC(psync) => psync.handle(args).await,
            RespCommandHandler::REPLCONF(replconf) => replconf.handle(args).await,
            RespCommandHandler::SET(set) => set.handle(args).await,
            RespCommandHandler::NULL => Ok(0),
            RespCommandHandler::ERR(handler) => handler.handle().await,
        }
    }

    pub fn err_handler(stream: &'a mut TcpStream, err: Error) -> RespCommandHandler<'a> {
        RespCommandHandler::ERR(ErrHandler::new(stream, err))
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

/// Special handler for writing error values into response stream.
pub struct ErrHandler<'a> {
    stream: &'a mut TcpStream,
    err: Error,
}

impl<'a> ErrHandler<'a> {
    fn new(stream: &'a mut TcpStream, err: Error) -> ErrHandler<'a> {
        ErrHandler { stream, err }
    }

    /// Create a SimpleError RESP value and write it to the response stream.
    async fn handle(&mut self) -> Result<usize> {
        let res = RespType::SimpleError(self.err.to_string());
        RespType::write_to_stream(self.stream, &res).await
    }
}
