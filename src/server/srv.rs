use crate::cli::args::Args;
use crate::commands::cmd::Cmd;
use crate::protocol::resp::handler::RespHandler;
use crate::protocol::resp::traits::{BytesWriter, RespReader, RespWriter};
use crate::protocol::resp::types::RespType;
use crate::replication::handshake;
use crate::server::info::{Role, ServerConfig};
use crate::storage::store::Store;
use anyhow::Result;
use bytes::BytesMut;
use log::info;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

/// TCP server for communicating with Nimblecache.
pub struct TcpServer<'a> {
    /// Arguments to be passed while instantiating the server.
    args: &'a Args,
    cfg: ServerConfig,
}

impl<'a> TcpServer<'a> {
    pub fn new(args: &Args) -> TcpServer {
        let srv_cfg = ServerConfig::new(args);
        let srv_cfg = match srv_cfg {
            Ok(si) => si,
            Err(e) => {
                panic!("Error while initializing server {}", e)
            }
        };

        TcpServer { args, cfg: srv_cfg }
    }

    /// Start listening to the post specified in the program arguments [Self::Args].
    /// If no ports are specified, it will default to port 6379.
    pub async fn start(self) {
        let server_config = self.cfg.clone();
        let role = server_config.role;
        let master = server_config.master;
        match role {
            Role::MASTER => {
                info!("Assuming role as master");
            }
            Role::SLAVE => {
                let master = &master.unwrap();
                info!("Assuming role as slave of {}:{}", master.host, master.port);

                // Replica server (slave) should perform handshake with master
                match handshake::Handshake::start(master.clone()).await {
                    Ok(_) => {
                        info!("Handshake success")
                    }
                    Err(e) => {
                        panic!("Error while performing handshake. Error: {}", e)
                    }
                };
            }
        }

        let server_config_arc = Arc::new(self.cfg.clone());

        let addr = format!("127.0.0.1:{}", self.args.port);
        info!("Starting TCP listener on port {}", self.args.port);

        let listener = TcpListener::bind(addr).await.unwrap();
        let storage = Store::new_simple_map();
        let storage_arc = Arc::new(storage);

        loop {
            let stream = listener.accept().await;
            let storage_arc = Arc::clone(&storage_arc);
            let server_config_arc = Arc::clone(&server_config_arc);

            match stream {
                Ok((mut stream, _)) => {
                    info!(
                        "accepted new connection from: {:?}",
                        stream.peer_addr().unwrap()
                    );

                    tokio::spawn(async move {
                        let mut resp_handler = RespHandler::new(&mut stream, 512);
                        loop {
                            let resp_command: Result<(Option<RespType>, _)> =
                                resp_handler.read().await;
                            let resp_command = match resp_command {
                                Ok((cmd, _)) => match cmd {
                                    None => break,
                                    Some(cmd) => cmd,
                                },
                                Err(_) => {
                                    panic!("Error reading the RESP command")
                                }
                            };
                            let cmd_response = Cmd::execute(
                                &resp_command,
                                Arc::as_ref(&storage_arc),
                                Arc::as_ref(&server_config_arc),
                            );
                            let (res, payload_bytes) = cmd_response.resp_output();

                            // If available, add the bytes payload in the response
                            if payload_bytes.is_none() {
                                resp_handler.write(&res).await.unwrap();
                            } else {
                                let mut byte_data = BytesMut::from(res.serialize().as_bytes());
                                byte_data.extend_from_slice(payload_bytes.unwrap().as_ref());
                                resp_handler.write_bytes(byte_data.as_ref()).await.unwrap();
                            }
                        }
                    });
                }
                Err(e) => {
                    panic!("error: {}", e)
                }
            };
        }
    }
}
