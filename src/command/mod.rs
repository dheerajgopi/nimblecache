// src/command/mod.rs

use core::fmt;

use ping::Ping;

use crate::resp::types::RespType;

mod ping;

/// Represents the supported Nimblecache commands.
#[derive(Debug, Clone)]
pub enum Command {
    /// The PING command.
    Ping(Ping),
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
    /// # Returns
    ///
    /// The result of the command execution as a `RespType`.
    pub fn execute(&self) -> RespType {
        match self {
            Command::Ping(ping) => ping.apply(),
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
