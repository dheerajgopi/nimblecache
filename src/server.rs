use anyhow::{Error, Result};
use log::error;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

use crate::{handler::FrameHandler, resp::frame::RespCommandFrame};

/// Represents a TCP server that listens for and handles RESP commands.
#[derive(Debug)]
pub struct Server {
    /// The TCP listener for accepting incoming connections.
    listener: TcpListener,
}

impl Server {
    /// Creates a new `Server` instance.
    pub fn new(listener: TcpListener) -> Server {
        Server { listener }
    }

    /// Starts listening for incoming connections and handles them.
    ///
    /// This method runs in an infinite loop, accepting new connections and spawning
    /// a new task to handle each one.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the operation succeeded or failed.
    ///
    /// # Errors
    ///
    /// This method will return an error if there's an issue with accepting connections.
    /// Note that it will panic if it encounters an error while accepting a connection.
    pub async fn listen(&mut self) -> Result<()> {
        loop {
            let sock = match self.accept_conn().await {
                Ok(stream) => stream,
                Err(e) => {
                    error!("{}", e);
                    panic!("Error accepting connection");
                }
            };

            let resp_command_frame = Framed::new(sock, RespCommandFrame::new());

            tokio::spawn(async move {
                let handler = FrameHandler::new(resp_command_frame);
                if let Err(e) = handler.handle().await {
                    error!("Failed to handle command: {}", e);
                }
            });
        }
    }

    /// Accepts a new TCP connection.
    ///
    /// This method attempts to accept a new connection from the TCP listener.
    ///
    /// # Returns
    ///
    /// A `Result` containing the accepted `TcpStream` if successful.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue accepting the connection.
    async fn accept_conn(&mut self) -> Result<TcpStream> {
        loop {
            match self.listener.accept().await {
                Ok((sock, _)) => return Ok(sock),
                Err(e) => return Err(Error::from(e)),
            }
        }
    }
}
