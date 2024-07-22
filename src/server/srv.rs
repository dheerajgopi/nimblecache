use crate::cli::args::Args;
use crate::commands::handler::RespCommandHandler;
use crate::protocol::resp::types::RespType;
use crate::replication::handshake;
use crate::server::info::{Role, ServerConfig};
use crate::storage::store::Store;
use bytes::BytesMut;
use log::{info, warn};
use std::sync::Arc;
use tokio::net::TcpListener;

const EMPTY_ARGS: Vec<RespType> = vec![];

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
                        loop {
                            let buffer = BytesMut::with_capacity(512);
                            let resp_handler = RespCommandHandler::from(
                                &mut stream,
                                buffer,
                                Arc::as_ref(&storage_arc),
                                Arc::as_ref(&server_config_arc),
                            )
                            .await;
                            let (mut resp_handler, args) = match resp_handler {
                                Ok((h, args)) => (h, args),
                                Err(e) => {
                                    warn!("Error reading the request: {}", e);
                                    (RespCommandHandler::err_handler(&mut stream, e), EMPTY_ARGS)
                                }
                            };

                            // NULL handler is used to skip the loop in case 0 bytes are read from the stream
                            match resp_handler {
                                RespCommandHandler::NULL => continue,
                                _ => {}
                            }

                            let args = args.iter().map(|a| a).collect::<Vec<&RespType>>();
                            let args = args.as_slice();
                            resp_handler.handle(args).await.unwrap();
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
