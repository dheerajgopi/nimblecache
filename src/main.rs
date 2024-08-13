mod command;
mod handler;
mod replication;
mod resp;
mod server;
mod storage;

use crate::server::Server;
use anyhow::{anyhow, Result};
use clap::Parser;
use log::{error, info};
use rand::distributions::{Alphanumeric, DistString};
use replication::Replication;
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
    /// Port to be bound to Nimblecache server
    #[arg(long)]
    port: Option<u16>,
    /// Specify which role is to be assumed by the server (master/slave)
    #[arg(long = "replicaof", default_value = "master")]
    pub replica_of: String,
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

    let replication_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 40);
    let master_host_port = match cli.parse_master_host_port() {
        Ok(hp) => hp,
        Err(e) => panic!("{}", e),
    };

    let replication = Replication::new(replication_id, master_host_port);

    // initialize storage
    let shared_storage = storage::db::Storage::new(storage::db::DB::new());

    info!("Started TCP listener on port {}", port);

    let mut server = Server::new(listener, shared_storage, replication);
    tokio::select! {
        res = server.listen() => {
            if let Err(err) = res {
                error!("failed to process the request: {}", err);
            }
        }
    }

    Ok(())
}

impl Cli {
    /// Parse master host and port from the "replicaof" CLI argument.
    /// If value of replicaof = "master", the master host and port wont be set.
    fn parse_master_host_port(&self) -> Result<Option<(String, u16)>> {
        let host_port_str = self.replica_of.clone();
        if host_port_str.to_lowercase().trim() == "master" {
            return Ok(None);
        }

        let mut split = host_port_str.split_whitespace();

        let host = match split.next() {
            Some(h) => h,
            None => {
                return Err(anyhow!("Invalid value for replicaof. replicaof should be in '<MASTER_HOST> <MASTER_PORT>' format"));
            }
        };

        let port = match split.next() {
            Some(p) => p,
            None => {
                return Err(anyhow!("Master port is not specified in replicaof"));
            }
        };

        let port = port.parse::<u16>();
        let port = match port {
            Ok(p) => p,
            Err(_) => {
                return Err(anyhow!("Invalid value for master port in replicaof"));
            }
        };

        Ok(Some((host.to_string(), port)))
    }
}
