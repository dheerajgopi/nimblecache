mod protocol;
mod commands;

use tokio::net::TcpListener;
use anyhow::Result;
use crate::commands::parser::get_command;
use crate::commands::traits::CommandExecutor;
use crate::protocol::resp::resp2::Resp2Handler;
use crate::protocol::resp::traits::{RespReader, RespWriter};
use crate::protocol::resp::datatypes::DataType;

#[tokio::main]
async fn main() {
    println!("Starting TCP listener on port 6379");

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((mut stream, _)) => {
                println!("accepted new connection from: {:?}", stream.peer_addr().unwrap());

                tokio::spawn(async move {
                    let mut resp_handler = Resp2Handler::new(&mut stream, 512);
                    loop {
                        let resp_command: Result<Option<DataType>> = resp_handler.read().await;
                        let resp_command = match resp_command {
                            Ok(cmd) => {
                                match cmd {
                                    None => {
                                        break
                                    }
                                    Some(cmd) => {
                                        cmd
                                    }
                                }
                            }
                            Err(_) => {
                                panic!("Error reading the RESP command")
                            }
                        };
                        let cmd = get_command(&resp_command);
                        let (cmd_executor, args) = match cmd {
                            Ok(cmd) => {
                                (cmd.0, cmd.1)
                            }
                            Err(_) => {
                                panic!("Invalid command")
                            }
                        };

                        let args = args.iter().map(|a| a).collect::<Vec<&DataType>>();
                        let res = cmd_executor.execute(args.as_slice());
                        let res = match res {
                            Ok(r) => {
                                r
                            }
                            Err(_) => {
                                panic!("Command execution failed")
                            }
                        };

                        resp_handler.write(&res).await.unwrap();
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
