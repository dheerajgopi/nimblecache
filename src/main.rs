mod cli;
mod commands;
mod protocol;
mod storage;

use crate::cli::args::Args;
use crate::commands::cmd::Cmd;
use crate::protocol::resp::handler::RespHandler;
use crate::protocol::resp::traits::{RespReader, RespWriter};
use crate::protocol::resp::types::RespType;
use crate::storage::store::Store;
use anyhow::Result;
use clap::Parser;
use env_logger;
use log::{error, info};
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let cli_args = Args::parse();

    // init logger
    env_logger::init();

    let addr = format!("127.0.0.1:{}", cli_args.port);
    info!("Starting TCP listener on port {}", cli_args.port);

    let listener = TcpListener::bind(addr).await.unwrap();
    let storage = Store::new_simple_map();
    let storage_arc = Arc::new(storage);

    loop {
        let stream = listener.accept().await;
        let storage_arc = Arc::clone(&storage_arc);

        match stream {
            Ok((mut stream, _)) => {
                info!(
                    "accepted new connection from: {:?}",
                    stream.peer_addr().unwrap()
                );

                tokio::spawn(async move {
                    let mut resp_handler = RespHandler::new(&mut stream, 512);
                    loop {
                        let resp_command: Result<Option<RespType>> = resp_handler.read().await;
                        let resp_command = match resp_command {
                            Ok(cmd) => match cmd {
                                None => break,
                                Some(cmd) => cmd,
                            },
                            Err(_) => {
                                panic!("Error reading the RESP command")
                            }
                        };
                        let res = Cmd::execute(&resp_command, Arc::as_ref(&storage_arc));

                        resp_handler.write(&res).await.unwrap();
                    }
                });
            }
            Err(e) => {
                error!("error: {}", e);
            }
        }
    }
}
