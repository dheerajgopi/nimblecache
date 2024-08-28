// src/server.rs

// anyhow provides the Error and Result types for convenient error handling
use anyhow::{Error, Result};

// log crate provides macros for logging at various levels (error, warn, info, debug, trace)
use log::error;

use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

use crate::{handler::FrameHandler, resp::frame::RespCommandFrame};

/// The Server struct holds the tokio TcpListener which listens for
/// incoming TCP connections.
#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Creates a new Server instance with the given TcpListener.
    pub fn new(listener: TcpListener) -> Server {
        Server { listener }
    }

    /// Runs the server in an infinite loop, continuously accepting and handling
    /// incoming connections.
    pub async fn run(&mut self) -> Result<()> {
        loop {
            // accept a new TCP connection.
            // If successful the corresponding TcpStream is stored
            // in the variable `sock`, else a panic will occur.
            let sock = match self.accept_conn().await {
                Ok(stream) => stream,
                // Log the error and panic if there is an issue accepting a connection.
                Err(e) => {
                    error!("{}", e);
                    panic!("Error accepting connection");
                }
            };

            // Use RespCommandFrame codec to read incoming TCP messages as Redis command frames,
            // and to write RespType values into outgoing TCP messages.
            let resp_command_frame = Framed::with_capacity(sock, RespCommandFrame::new(), 8 * 1024);

            // Spawn a new asynchronous task to handle the connection.
            // This allows the server to handle multiple connections concurrently.
            tokio::spawn(async move {
                let handler = FrameHandler::new(resp_command_frame);
                if let Err(e) = handler.handle().await {
                    error!("Failed to handle command: {}", e);
                }
            });
        }
    }

    /// Accepts a new incoming TCP connection and returns the corresponding
    /// tokio TcpStream.
    async fn accept_conn(&mut self) -> Result<TcpStream> {
        loop {
            // Wait for an incoming connection.
            // The `accept()` method returns a tuple of (TcpStream, SocketAddr),
            // but we only need the TcpStream.
            match self.listener.accept().await {
                // Return the TcpStream if a connection is successfully accepted.
                Ok((sock, _)) => return Ok(sock),
                // Return an error if there is an issue accepting a connection.
                Err(e) => return Err(Error::from(e)),
            }
        }
    }
}
