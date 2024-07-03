mod protocol;
mod commands;
mod storage;

use tokio::net::TcpListener;
use anyhow::Result;
use crate::commands::cmd::Cmd;
use crate::protocol::resp::resp2::Resp2Handler;
use crate::protocol::resp::traits::{RespReader, RespWriter};
use crate::protocol::resp::types::RespType;
use crate::storage::store::Store;

#[tokio::main]
async fn main() {
    println!("Starting TCP listener on port 6379");

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let storage = &mut Store::new_simple_map();

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((mut stream, _)) => {
                println!("accepted new connection from: {:?}", stream.peer_addr().unwrap());

                tokio::spawn(async move {
                    let mut resp_handler = Resp2Handler::new(&mut stream, 512);
                    loop {
                        let resp_command: Result<Option<RespType>> = resp_handler.read().await;
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
                        let res = Cmd::execute(&resp_command, storage);

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
