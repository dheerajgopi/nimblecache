use anyhow::Result;
use futures::{SinkExt, StreamExt};
use log::{error, warn};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::command::pipelining::MultiCommand;
use crate::replication::Replication;
use crate::resp::types::RespType;
use crate::storage::db::DB;
use crate::{command::Command, resp::frame::RespCommandFrame};

/// Handles RESP command frames over a single TCP connection.
pub struct FrameHandler {
    /// The framed connection using `RespCommandFrame` as the codec.
    conn: Framed<TcpStream, RespCommandFrame>,
}
impl FrameHandler {
    /// Creates a new `FrameHandler` instance.
    pub fn new(conn: Framed<TcpStream, RespCommandFrame>) -> FrameHandler {
        FrameHandler { conn }
    }

    /// Handles incoming RESP command frames.
    ///
    /// This method continuously reads command frames from the connection,
    /// processes them, and sends back the responses. It continues until
    /// an error occurs or the connection is closed.
    ///
    /// The server's behavior depends on whether a `MULTI` command has been issued.
    ///
    /// ## No MULTI Command Issued
    ///
    /// If no `MULTI` command has been issued, each command is executed
    /// immediately and its response is sent back.
    ///
    /// ## MULTI Command Issued
    ///
    /// If a `MULTI` command has been issued, the method will enter a transaction
    /// mode. In this mode, all subsequent commands will be queued until an
    /// `EXEC` command is received. When `EXEC` is called, all the queued
    /// commands are executed, and the array of responses is sent back.
    ///
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// * `replication` - Server replication.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the operation succeeded or failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if there's an issue with reading
    /// from or writing to the connection.
    pub async fn handle(mut self, db: &DB, replication: &Replication) -> Result<()> {
        // commands are queued here if MULTI command was issued
        let mut multicommand = MultiCommand::new();

        while let Some(resp_cmd) = self.conn.next().await {
            match resp_cmd {
                Ok(cmd_frame) => {
                    // Read the command from the frame.
                    let resp_cmd = Command::from_resp_command_frame(cmd_frame);

                    // If command is parsed successfully, execute it and get the RESP response,
                    // otherwise set a SimpleError RESP value as the response.
                    let response = match resp_cmd {
                        Ok(cmd) => match cmd {
                            // Initialize pipeline if MULTI command is issued
                            Command::Multi => {
                                let init_multicommand = &mut multicommand.init();
                                match init_multicommand {
                                    Ok(_) => cmd.execute(db, replication),
                                    Err(e) => RespType::SimpleError(format!("{}", e)),
                                }
                            }
                            // Execute all commands in pipeline if EXEC command is issued
                            Command::Exec => {
                                if multicommand.is_active() {
                                    multicommand.exec(db, replication).await
                                } else {
                                    RespType::SimpleError(String::from("EXEC without MULTI"))
                                }
                            }
                            Command::Discard => {
                                if multicommand.is_active() {
                                    multicommand.discard();
                                    cmd.execute(db, replication)
                                } else {
                                    RespType::SimpleError(String::from("DISCARD without MULTI"))
                                }
                            }
                            Command::Psync(psync) => {
                                let res = psync.apply(replication);
                                // Write the RESP response into the TCP stream.
                                if let Err(e) = self.conn.send(res).await {
                                    error!("Error sending response: {}", e);
                                    break;
                                };
                                // flush the buffer into the TCP stream.
                                self.conn.flush().await?;

                                replication.add_replica_peer(self.conn.into_inner()).await;

                                break;
                            }
                            _ => {
                                // Queue commands if pipeline is active, else execute the command
                                if multicommand.is_active() {
                                    multicommand.add_command(cmd);
                                    RespType::SimpleString(String::from("QUEUED"))
                                } else {
                                    let res = cmd.execute(db, replication);
                                    if let Some(replica_cmd) = cmd.replication_cmd() {
                                        replication.write_to_replicas(replica_cmd).await;
                                    };

                                    res
                                }
                            }
                        },
                        Err(e) => {
                            if multicommand.is_active() {
                                multicommand.discard();
                            }
                            RespType::SimpleError(format!("{}", e))
                        }
                    };

                    // Write the RESP response into the TCP stream.
                    if let Err(e) = self.conn.send(response).await {
                        error!("Error sending response: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Error reading the request: {}", e);
                    break;
                }
            };

            // flush the buffer into the TCP stream.
            self.conn.flush().await?;
        }

        Ok(())
    }

    /// Handles incoming RESP command frames from the replication stream.
    ///
    /// This method continuously reads command frames from the master's replication stream and
    /// processes them. The responses are not sent back to the master.
    ///
    /// # Arguments
    ///
    /// * `db` - The database where the key and values are stored.
    ///
    /// * `replication` - Server replication.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the operation succeeded or failed.
    pub async fn handle_replication_stream(
        mut self,
        db: &DB,
        replication: &Replication,
    ) -> Result<()> {
        while let Some(resp_cmd) = self.conn.next().await {
            match resp_cmd {
                Ok(cmd_frame) => {
                    // Read the command from the frame.
                    let resp_cmd = Command::from_resp_command_frame(cmd_frame);

                    // If command is parsed successfully, execute it.
                    match resp_cmd {
                        Ok(cmd) => {
                            cmd.execute(db, replication);
                            if let Some(replica_cmd) = cmd.replication_cmd() {
                                replication.write_to_replicas(replica_cmd).await;
                            };
                        }
                        Err(e) => {
                            warn!("Error executing the command from replication stream: {}", e);
                        }
                    };
                }
                Err(e) => {
                    error!("Error replicating the request: {}", e);
                }
            };
        }

        Ok(())
    }
}
