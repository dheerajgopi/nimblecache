// src/main.rs

// Include the server module defined in server.rs
mod handler;
mod resp;
mod server;

use crate::server::Server;
use anyhow::Result;
use log::info;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger.
    // This sets up logging based on the RUST_LOG environment variable
    env_logger::init();

    // Define the address and port for the TCP server to listen on
    // Here we're using localhost (127.0.0.1) and port 6379 (commonly used for Redis)
    let addr = format!("127.0.0.1:{}", 6379);

    // Attempt to bind the TCP listener to the specified address and port
    let listener = match TcpListener::bind(&addr).await {
        // If successful, return the TcpListener
        Ok(tcp_listener) => {
            info!("TCP listener started on port 6379");
            tcp_listener
        }
        // If there is an error, panic and print the error message
        // This could happen if the port is already in use, for example
        Err(e) => panic!("Could not bind the TCP listener to {}. Err: {}", &addr, e),
    };

    // Create a new instance of the Server with the bound TcpListener
    let mut server = Server::new(listener);

    // Run the server to start accepting and handling connections
    // This will run indefinitely until the program is terminated
    server.run().await?;

    // This Ok(()) is technically unreachable as server.run() loops infinitely,
    // but it's needed to satisfy the Result return type of main()
    Ok(())
}
