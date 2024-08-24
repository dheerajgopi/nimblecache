// src/server.rs

// anyhow provides the Error and Result types for convenient error handling
use anyhow::{Error, Result};

// log crate provides macros for logging at various levels (error, warn, info, debug, trace)
use log::error;

use tokio::{
    // AsyncWriteExt trait provides asynchronous write methods like write_all
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

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
            let mut sock = match self.accept_conn().await {
                Ok(stream) => stream,
                // Log the error and panic if there is an issue accepting a connection.
                Err(e) => {
                    error!("{}", e);
                    panic!("Error accepting connection");
                }
            };

            // Spawn a new asynchronous task to handle the connection.
            // This allows the server to handle multiple connections concurrently.
            tokio::spawn(async move {
                // Write a "Hello!" message to the client.
                if let Err(e) = &mut sock.write_all("Hello!".as_bytes()).await {
                    // Log the error and panic if there is an issue writing the response.
                    error!("{}", e);
                    panic!("Error writing response")
                }
                // The connection is closed automatically when `sock` goes out of scope.
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