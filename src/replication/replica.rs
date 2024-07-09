use crate::commands::ping::Ping;
use crate::commands::traits::CommandBuilder;
use crate::protocol::resp::handler::RespHandler;
use crate::protocol::resp::traits::{RespReader, RespWriter};
use crate::server::info::{Role, ServerInfo};
use anyhow::{anyhow, Result};
use log::info;
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
        let mut resp_handler = RespHandler::new(&mut stream, 512);
        let req = resp_handler.write(&Ping::build(None)).await;
        match req {
            Ok(b) => {
                info!("Bytes written to master server during handshake: {}", b)
            }
            Err(e) => return Err(anyhow!("Failed to PING master during handshake: {}", e)),
        }
        let res = resp_handler.read().await;
        match res {
            Ok(resp) => match resp {
                None => return Err(anyhow!("Received null PONG from master during handshake")),
                Some(pong) => {
                    info!(
                        "Received response for PING during handshake: {}",
                        pong.serialize()
                    )
                }
            },
            Err(e) => {
                return Err(anyhow!(
                    "Failed to receive PONG from master during handshake: {}",
                    e
                ))
            }
        };

        return Ok(true);
    }
}
