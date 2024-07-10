use crate::commands::ping::Ping;
use crate::commands::replconf::Replconf;
use crate::commands::traits::CommandBuilder;
use crate::protocol::resp::handler::RespHandler;
use crate::protocol::resp::traits::{RespReader, RespWriter};
use crate::protocol::resp::types::RespType;
use crate::server::info::{Role, ServerInfo};
use anyhow::{anyhow, Result};
use log::info;
use std::vec;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// This is used for operations related to replica server like:
/// - Handshake with master server.
pub struct Replica<'a> {
    /// Server info
    svr_info: &'a ServerInfo,
}

impl<'a> Replica<'a> {
    pub fn new(svr_info: &ServerInfo) -> Replica {
        Replica { svr_info }
    }

    /// Perform handshake with master server during the replica server initialization.
    ///
    /// Step 1: Send PING command to master
    ///
    /// Step 2: Send 2 REPLCONF commands to master - `REPLCONF listening-port <PORT>` and `REPLCONF capa psync2`,
    ///         where `<PORT>` is the port where the replica is listening.
    pub async fn handshake(&self) -> Result<bool> {
        // get master host and port
        let slave_info = match &self.svr_info.role {
            Role::MASTER(_) => {
                return Err(anyhow!(
                    "Server with master role cannot be added as a replica"
                ));
            }
            Role::SLAVE(s) => s,
        };

        // try opening a connection to master server
        let stream = TcpStream::connect(format!(
            "{}:{}",
            slave_info.master_host, slave_info.master_port
        ))
        .await;
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => return Err(anyhow!("Handshake failed with error: {}", e)),
        };

        // Try PINGing the master server
        let ping_res = self.ping_master(&mut stream).await;
        match ping_res {
            Ok(pong) => {
                info!(
                    "Received response for PING during handshake: {}",
                    pong.serialize()
                );
            }
            Err(e) => {
                return Err(e);
            }
        }

        // flush stream
        let flush = stream.flush().await;
        match flush {
            Ok(_) => {}
            Err(_) => {}
        }

        // Try REPLCONF listening-port with the master server
        let replconf_args = vec![
            RespType::BulkString("listening-port".to_string()),
            RespType::BulkString(format!("{}", self.svr_info.port)),
        ];
        info!(
            "Sending 'REPLCONF listening-port {}' request to master",
            self.svr_info.port
        );
        let replconf_res = self.replconf_master(&mut stream, replconf_args).await;
        match replconf_res {
            Ok(ok) => {
                info!(
                    "Received response for 'RESPCONF listening-port' during handshake: {}",
                    ok.serialize()
                );
            }
            Err(e) => {
                return Err(e);
            }
        }

        // flush stream
        let flush = stream.flush().await;
        match flush {
            Ok(_) => {}
            Err(_) => {}
        }

        // Try REPLCONF capa psync2 with the master server
        let replconf_args = vec![
            RespType::BulkString("capa".to_string()),
            RespType::BulkString("psync2".to_string()),
        ];
        info!("Sending 'REPLCONF capa psync2' request to master");
        let replconf_res = self.replconf_master(&mut stream, replconf_args).await;
        match replconf_res {
            Ok(ok) => {
                info!(
                    "Received response for 'RESPCONF capa psync2' during handshake: {}",
                    ok.serialize()
                );
            }
            Err(e) => {
                return Err(e);
            }
        }

        return Ok(true);
    }

    async fn ping_master(&self, stream: &mut TcpStream) -> Result<RespType> {
        let mut resp_handler = RespHandler::new(stream, 512);
        let req = resp_handler.write(&Ping::build(None)).await;
        match req {
            Ok(b) => {
                info!(
                    "Bytes written to master server during handshake PING: {}",
                    b
                )
            }
            Err(e) => return Err(anyhow!("Failed to PING master during handshake: {}", e)),
        }
        let res = resp_handler.read().await;
        match res {
            Ok(resp) => match resp {
                None => return Err(anyhow!("Received null PONG from master during handshake")),
                Some(pong) => Ok(pong),
            },
            Err(e) => {
                return Err(anyhow!(
                    "Failed to receive PONG from master during handshake: {}",
                    e
                ))
            }
        }
    }

    async fn replconf_master(
        &self,
        stream: &mut TcpStream,
        replconf_args: Vec<RespType>,
    ) -> Result<RespType> {
        let mut resp_handler = RespHandler::new(stream, 512);
        let req = resp_handler
            .write(&Replconf::build(Some(
                replconf_args.iter().collect::<Vec<&RespType>>().as_slice(),
            )))
            .await;
        match req {
            Ok(b) => {
                info!(
                    "Bytes written to master server during handshake REPLCONF: {}",
                    b
                )
            }
            Err(e) => {
                return Err(anyhow!(
                    "Failed to request 'REPLCONF' to master during handshake: {}",
                    e
                ))
            }
        }
        let res = resp_handler.read().await;
        match res {
            Ok(resp) => match resp {
                None => {
                    return Err(anyhow!(
                        "Received null response for 'REPLCONF' from master during handshake"
                    ))
                }
                Some(ok) => {
                    if Self::is_ok(&ok) {
                        Ok(ok)
                    } else {
                        return Err(anyhow!("Received non-OK response for 'REPLCONF' from master during handshake: {}", ok.serialize()));
                    }
                }
            },
            Err(e) => {
                return Err(anyhow!(
                    "Failed to receive response for 'REPLCONF' from master during handshake: {}",
                    e
                ))
            }
        }
    }

    fn is_ok(res: &RespType) -> bool {
        match res {
            RespType::SimpleString(s) => s == "OK",
            _ => false,
        }
    }
}
