use crate::commands::traits::CommandExecutor;
use crate::commands::{echo, get, info, ping, set};
use crate::protocol::resp::types::RespType;
use crate::storage::store::Store;
use anyhow::{anyhow, Result};

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
    pub fn execute(resp_val: &RespType, store: &Store) -> RespType {
        let cmd_name_and_args = Cmd::extract_command_name_and_args(resp_val);
        let (cmd_name, args) = match cmd_name_and_args {
            Ok(cmd) => (cmd.0, cmd.1),
            Err(e) => return RespType::SimpleError(format!("(error) {:?}", e)),
        };
        let args = args.iter().map(|a| a).collect::<Vec<&RespType>>();
        let args = args.as_slice();

        match cmd_name.to_uppercase().as_str() {
            "ECHO" => echo::Echo {}.execute(args),
            "GET" => get::Get::new(store).execute(args),
            "INFO" => info::Info {}.execute(args),
            "PING" => ping::Ping {}.execute(args),
            "SET" => set::Set::new(store).execute(args),
            _ => RespType::SimpleError(format!("(error) unknown command '{:?}'", cmd_name)),
        }
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
