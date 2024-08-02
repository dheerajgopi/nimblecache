mod server;

use crate::server::Server;
use anyhow::Result;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = format!("127.0.0.1:{}", 6379);
    let listener = match TcpListener::bind(&addr).await {
        Ok(tcp_listener) => tcp_listener,
        Err(e) => panic!("Could not bind the TCP listener to {}. Err: {}", &addr, e),
    };

    let mut server = Server::new(listener);
    server.run().await?;

    Ok(())
}
