use core::fmt;

use get::Get;
use info::Info;
use lpush::LPush;
use lrange::LRange;
use ping::Ping;
use psync::Psync;
use rpush::RPush;
use set::Set;

use crate::{replication::Replication, resp::types::RespType, storage::db::DB};

mod get;
mod info;
mod lpush;
mod lrange;
pub mod ping;
pub mod pipelining;
pub mod psync;
mod rpush;
mod set;

/// Represents the supported Nimblecache commands.
#[derive(Debug, Clone)]
pub enum Command {
    /// The PING command.
    Ping(Ping),
    /// The INFO command.
    Info(Info),
    /// The MULTI command.
    Multi,
    /// The EXEC command.
    Exec,
    /// The DISCARD command.
    Discard,
    /// The SET command.
    Set(Set),
    /// The GET command.
    Get(Get),
    /// The LPUSH command.
    LPush(LPush),
    /// The RPUSH command.
    RPush(RPush),
    /// The LRANGE command.
    LRange(LRange),
    /// The PSYNC command.
    Psync(Psync),
}

impl Command {
    /// Attempts to parse a Nimblecache command from a RESP command frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - A vector of `RespType` representing the command and its arguments.
    /// The first item is always the command name, and the rest are its arguments.
    ///
    /// # Returns
    ///
    /// * `Ok(Command)` if parsing succeeds.
    /// * `Err(CommandError)` if parsing fails.
    pub fn from_resp_command_frame(frame: Vec<RespType>) -> Result<Command, CommandError> {
        let (cmd_name, args) = frame.split_at(1);
        let cmd_name = match &cmd_name[0] {
            RespType::BulkString(s) => s.clone(),
            _ => return Err(CommandError::InvalidFormat),
        };

        let cmd = match cmd_name.to_lowercase().as_str() {
            "ping" => Command::Ping(Ping::with_args(Vec::from(args))?),
            "info" => Command::Info(Info::with_args(Vec::from(args))?),
            "multi" => Command::Multi,
            "exec" => Command::Exec,
            "discard" => Command::Discard,
            "set" => {
                let cmd = Set::with_args(Vec::from(args));
                match cmd {
                    Ok(cmd) => Command::Set(cmd),
                    Err(e) => return Err(e),
                }
            }
            "get" => {
                let cmd = Get::with_args(Vec::from(args));
                match cmd {
                    Ok(cmd) => Command::Get(cmd),
                    Err(e) => return Err(e),
                }
            }
            "lpush" => {
                let cmd = LPush::with_args(Vec::from(args));
                match cmd {
                    Ok(cmd) => Command::LPush(cmd),
                    Err(e) => return Err(e),
                }
            }
            "rpush" => {
                let cmd = RPush::with_args(Vec::from(args));
                match cmd {
                    Ok(cmd) => Command::RPush(cmd),
                    Err(e) => return Err(e),
                }
            }
            "lrange" => {
                let cmd = LRange::with_args(Vec::from(args));
                match cmd {
                    Ok(cmd) => Command::LRange(cmd),
                    Err(e) => return Err(e),
                }
            }
            "psync" => {
                let cmd = Psync::with_args(Vec::from(args));
                match cmd {
                    Ok(cmd) => Command::Psync(cmd),
                    Err(e) => return Err(e),
                }
            }
            _ => {
                return Err(CommandError::UnknownCommand(ErrUnknownCommand {
                    cmd: cmd_name,
                }));
            }
        };

        Ok(cmd)
    }

    /// Executes the Nimblecache command.
    ///
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// * `replication` - Server replication.
    ///
    /// # Returns
    ///
    /// The result of the command execution as a `RespType`.
    pub fn execute(&self, db: &DB, replication: &Replication) -> RespType {
        match self {
            Command::Ping(ping) => ping.apply(),
            Command::Info(info) => info.apply(replication),
            // MULTI calls are handled inside FrameHandler.handle since it involves command queueing.
            Command::Multi => RespType::SimpleString(String::from("OK")),
            // EXEC calls are handled inside FrameHandler.handle too, since it involves executing queued commands.
            Command::Exec => RespType::NullBulkString,
            // DISCARD calls are handled inside FrameHandler.handle too, since it involves discarding queued commands.
            Command::Discard => RespType::SimpleString(String::from("OK")),
            Command::Set(set) => set.apply(db),
            Command::Get(get) => get.apply(db),
            Command::LPush(lpush) => lpush.apply(db),
            Command::RPush(rpush) => rpush.apply(db),
            Command::LRange(lrange) => lrange.apply(db),
            Command::Psync(psync) => psync.apply(replication),
        }
    }

    /// Builds the RESP command which is to be sent as part of replication stream.
    /// Returns None if the command is for READ operation.
    pub fn replication_cmd(&self) -> Option<RespType> {
        match self {
            Command::Set(set) => Some(set.build_command()),
            Command::LPush(lpush) => Some(lpush.build_command()),
            Command::RPush(rpush) => Some(rpush.build_command()),
            _ => None,
        }
    }
}

/// Represents all possible errors that can occur during command parsing and execution.
#[derive(Debug)]
pub enum CommandError {
    /// Indicates that the command format is invalid.
    InvalidFormat,
    /// Indicates that the command is unknown.
    UnknownCommand(ErrUnknownCommand),
    /// Represents any other error with a descriptive message.
    Other(String),
}

/// Represents an error for an unknown command.
#[derive(Debug)]
pub struct ErrUnknownCommand {
    /// The name of the unknown command.
    pub cmd: String,
}

impl std::error::Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::InvalidFormat => "Invalid command format".fmt(f),
            CommandError::UnknownCommand(e) => write!(f, "Unknown command: {}", e.cmd),
            CommandError::Other(msg) => msg.as_str().fmt(f),
        }
    }
}
