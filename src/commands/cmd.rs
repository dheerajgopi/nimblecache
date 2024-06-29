use crate::commands::traits::CommandExecutor;
use crate::commands::{echo, ping};
use crate::protocol::resp::datatypes::DataType;
use anyhow::{anyhow, Result};

pub struct Cmd {}

impl Cmd {
    pub fn execute(resp_val: &DataType) -> DataType {
        let cmd_name_and_args = Cmd::extract_command_name_and_args(resp_val);
        let (cmd_name, args) = match cmd_name_and_args {
            Ok(cmd) => {
                (cmd.0, cmd.1)
            }
            Err(e) => {
                return DataType::SimpleError(format!("(error) {:?}", e))
            }
        };
        let args = args.iter().map(|a| a).collect::<Vec<&DataType>>();
        let args = args.as_slice();

        match cmd_name.to_uppercase().as_str() {
            "PING" => {
                ping::Ping{}.execute(args)
            },
            "ECHO" => {
                echo::Echo{}.execute(args)
            },
            _ => {
                DataType::SimpleError(format!("(error) unknown command '{:?}'", cmd_name))
            }
        }
    }

    fn extract_command_name_and_args(resp_val: &DataType) -> Result<(String, Vec<DataType>)> {
        let resp_arr = match resp_val {
            DataType::Array(arr) => {
                arr
            }
            _ => {
                return Err(anyhow!("Invalid command format"))
            }
        };

        if resp_arr.len() == 0 {
            return Err(anyhow!("No commands are provided"))
        }

        let cmd_name = resp_arr.first().unwrap();
        let cmd_name = match cmd_name {
            DataType::BulkString(name) => {
                name
            }
            _ => {
                return Err(anyhow!("Invalid command format"))
            }
        };

        let args = resp_arr.into_iter().skip(1).map(|arg| arg.clone()).collect::<Vec<DataType>>();

        Ok((cmd_name.into(), args))
    }
}
