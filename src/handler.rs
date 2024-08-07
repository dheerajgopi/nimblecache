use anyhow::Result;
use futures::{SinkExt, StreamExt};
use log::error;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::resp::types::RespType;
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
    /// # Returns
    ///
    /// A `Result` indicating whether the operation succeeded or failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if there's an issue with reading
    /// from or writing to the connection.
    pub async fn handle(mut self) -> Result<()> {
        while let Some(resp_cmd) = self.conn.next().await {
            match resp_cmd {
                Ok(cmd_frame) => {
                    // Read the command from the frame.
                    let resp_cmd = Command::from_resp_command_frame(cmd_frame);

                    // If command is parsed successfully, execute it and get the RESP response,
                    // otherwise set a SimpleError RESP value as the response.
                    let response = match resp_cmd {
                        Ok(cmd) => cmd.execute(),
                        Err(e) => RespType::SimpleError(format!("{}", e)),
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
}
