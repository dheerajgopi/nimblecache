mod command;
mod handler;
mod resp;
mod server;
mod storage;

use crate::server::Server;
use anyhow::Result;
use clap::Parser;
use log::{error, info};
use tokio::net::TcpListener;

const DEFAULT_PORT: u16 = 6379;

#[derive(Debug, Parser)]
#[command(
    name = "nimblecache-server",
    version,
    author,
    about = "A RESP based in-memory cache"
)]
struct Cli {
    #[arg(long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let port = cli.port.unwrap_or(DEFAULT_PORT);

    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(tcp_listener) => tcp_listener,
        Err(e) => panic!("Could not bind the TCP listener to {}. Err: {}", &addr, e),
    };

    info!("Started TCP listener on port {}", port);

    let mut server = Server::new(listener);
    tokio::select! {
        res = server.listen() => {
            if let Err(err) = res {
                error!("failed to process the request: {}", err);
            }
        }
    }

    Ok(())
}
