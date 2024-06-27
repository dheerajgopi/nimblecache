use crate::commands::traits::CommandExecutor;
use crate::commands::ping;
use crate::protocol::resp::datatypes::DataType;
use anyhow::{anyhow, Result};

pub fn get_command(resp_val: &DataType) -> Result<(impl CommandExecutor, Vec<DataType>)> {
    let (cmd_name, args) = extract_command_name_and_args(resp_val)?;

    match cmd_name.as_str() {
        "PING" => {
            Ok((ping::Ping{}, args))
        }
        _ => {
            Err(anyhow!("Unknown command {:?}", cmd_name))
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
            name.to_uppercase()
        }
        _ => {
            return Err(anyhow!("Invalid command format"))
        }
    };

    let args = resp_arr.into_iter().skip(1).map(|arg| arg.clone()).collect::<Vec<DataType>>();

    Ok((cmd_name, args))
}
