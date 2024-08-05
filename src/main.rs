mod resp;
mod server;

use crate::server::Server;
use anyhow::Result;
use log::{error, info};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = format!("127.0.0.1:{}", 6379);
    let listener = match TcpListener::bind(&addr).await {
        Ok(tcp_listener) => tcp_listener,
        Err(e) => panic!("Could not bind the TCP listener to {}. Err: {}", &addr, e),
    };

    info!("Started TCP listener on port {}", 6379);

    let mut server = Server::new(listener);
    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!("failed to accept request: {}", err);
            }
        }
    }
    server.run().await?;

    Ok(())
}
