use std::sync::Arc;

use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use log::{error, info};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Framed};

use crate::command::ping::Ping;
use crate::command::psync::Psync;
use crate::resp::types::RespType;
use crate::{handler::FrameHandler, resp::frame::RespCommandFrame, storage::db::Storage};

use super::Replication;

/// This is used for replication stream listener related functionalities.
pub struct MasterServer {}

impl MasterServer {
    /// Perform the handshake process with the master server.
    /// The handshake process includes the following steps:
    ///
    /// - Send a PING request and validate for PONG response
    /// - Send 2 REPLCONF commands to master: `REPLCONF listening-port <PORT>` and `REPLCONF capa psync2`,
    ///         where `<PORT>` is the port where the replica is listening. This is not implemented as of now.
    /// - Send PSYNC <REPLICATION_ID> <OFFSET> command to master, and perform a full-resync if required. Resync
    ///         is not implemented as of now.
    pub async fn perform_handshake(stream: TcpStream) -> Result<TcpStream> {
        let mut handshake_frame = Framed::with_capacity(stream, BytesCodec::new(), 8 * 1024);

        // PING master server
        if let Err(e) = handshake_frame.send(Ping::build_command().to_bytes()).await {
            return Err(anyhow!(
                "Failed to send PING to master during handshake: {}",
                e
            ));
        };

        // validate PING response
        match handshake_frame.next().await.transpose() {
            Ok(res) => match res {
                Some(b) => {
                    let resp_val = RespType::new_simple_string(b);
                    match resp_val {
                        Ok((ss, _)) => {
                            if let RespType::SimpleString(s) = ss {
                                if s != "PONG" {
                                    return Err(anyhow!(
                                        "Invalid response for PING request to master during handshake",
                                    ));
                                }

                                info!("Successfully PINGed master server");
                            }
                        }
                        Err(e) => {
                            return Err(anyhow!(
                                "Error response for PING request to master during handshake: {}",
                                e
                            ))
                        }
                    }
                }
                None => {
                    return Err(anyhow!(
                        "No response for PING request to master during handshake"
                    ))
                }
            },
            Err(e) => {
                return Err(anyhow!(
                    "Failed to receive response for PING request to master during handshake: {}",
                    e
                ))
            }
        }

        // PSYNC master server
        let psync_cmd = Psync::new("?".into(), None);
        if let Err(e) = handshake_frame
            .send(psync_cmd.build_command().to_bytes())
            .await
        {
            return Err(anyhow!(
                "Failed to send PSYNC to master during handshake: {}",
                e
            ));
        };

        // validate PSYNC response
        match handshake_frame.next().await.transpose() {
            Ok(res) => match res {
                Some(b) => {
                    let resp_val = RespType::new_simple_string(b);
                    match resp_val {
                        Ok((ss, _)) => {
                            if let RespType::BulkString(s) = ss {
                                if !s.starts_with("FULLRESYNC") {
                                    return Err(anyhow!(
                                        "Invalid response for PSYNC request to master during handshake",
                                    ));
                                }

                                info!("Successfully PSYNCed master server");
                            }
                        }
                        Err(e) => {
                            return Err(anyhow!(
                                "Error response for PSYNC request to master during handshake: {}",
                                e
                            ))
                        }
                    }
                }
                None => {
                    return Err(anyhow!(
                        "No response for PSYNC request to master during handshake"
                    ))
                }
            },
            Err(e) => {
                return Err(anyhow!(
                    "Failed to receive response for PSYNC request to master during handshake: {}",
                    e
                ))
            }
        }

        // return the inner TcpStream used to perform the handshake
        Ok(handshake_frame.into_inner())
    }

    /// Listen to the replication stream from the master and execute the commands coming through
    /// the replication stream. It uses the same TCP stream which was used for the handshake process.
    pub async fn listen(
        stream: TcpStream,
        storage: Storage,
        replication: Replication,
    ) -> Result<()> {
        let db = storage.db().clone();
        let replication = Arc::new(replication.clone());

        // listen to the master server replication stream
        let resp_command_frame = Framed::with_capacity(stream, RespCommandFrame::new(), 8 * 1024);

        let handler = FrameHandler::new(resp_command_frame);
        info!("Initialize master server listener");
        if let Err(e) = handler
            .handle_replication_stream(db.as_ref(), replication.as_ref())
            .await
        {
            error!("Failed to handle command: {}", e);
        }

        Ok(())
    }
}
