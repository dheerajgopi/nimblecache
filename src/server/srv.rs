use crate::cli::args::Args;
use crate::commands::cmd::Cmd;
use crate::protocol::resp::handler::RespHandler;
use crate::protocol::resp::traits::{RespReader, RespWriter};
use crate::protocol::resp::types::RespType;
use crate::replication::replica::Replica;
use crate::server::info::{Role, ServerInfo};
use crate::storage::store::Store;
use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use tokio::net::TcpListener;

/// TCP server for communicating with Nimblecache.
pub struct TcpServer<'a> {
    /// Arguments to be passed while instantiating the server.
    args: &'a Args,
}

impl<'a> TcpServer<'a> {
    pub fn new(args: &Args) -> TcpServer {
        TcpServer { args }
    }

    /// Start listening to the post specified in the program arguments [Self::Args].
    /// If no ports are specified, it will default to port 6379.
    pub async fn start(&self) {
        // Assume master/slave role
        let role = match Role::from_str(self.args.replica_of.as_str()) {
            Ok(r) => r,
            Err(e) => {
                error!("error: {}", e);
                panic!("Error while starting the server. Error: {}", e)
            }
        };
        let server_info = ServerInfo::new(role, self.args.port);
        info!("Assuming role as {}", server_info.role);

        // Replica server (slave) should perform handshake with master
        match server_info.role {
            Role::MASTER(_) => {}
            Role::SLAVE(_) => {
                let replica = Replica::new(&server_info);
                match replica.handshake().await {
                    Ok(_) => {
                        info!("Handshake success")
                    }
                    Err(e) => {
                        panic!("Error while performing handshake. Error: {}", e)
                    }
                };
            }
        }

        let server_info_arc = Arc::new(server_info);

        let addr = format!("127.0.0.1:{}", self.args.port);
        info!("Starting TCP listener on port {}", self.args.port);

        let listener = TcpListener::bind(addr).await.unwrap();
        let storage = Store::new_simple_map();
        let storage_arc = Arc::new(storage);

        loop {
            let stream = listener.accept().await;
            let storage_arc = Arc::clone(&storage_arc);
            let server_info_arc = Arc::clone(&server_info_arc);

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
                            let res = Cmd::execute(
                                &resp_command,
                                Arc::as_ref(&storage_arc),
                                Arc::as_ref(&server_info_arc),
                            );

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
}
