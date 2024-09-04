mod command;
mod handler;
mod replication;
mod resp;
mod server;
mod storage;

use std::sync::Arc;

use crate::server::Server;
use anyhow::{anyhow, Error, Result};
use clap::Parser;
use log::{error, info};
use rand::distributions::{Alphanumeric, DistString};
use replication::{master::MasterServer, Replication};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

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
async fn accept_conn(listener: &TcpListener) -> Result<TcpStream> {
    loop {
        match listener.accept().await {
            Ok((sock, _)) => return Ok(sock),
            Err(e) => return Err(Error::from(e)),
        }
    }
}

/// What's happening in `main`?
///
/// * Start 2 tokio runtimes - one for accepting connections,
/// another for handling the commands from these TCP connections. The TCP streams
/// from acceptor runtime is passed to command handler runtime using a channel.
/// Note that values global to the application are passed to the tokio runtimes via separate Arcs.
///
/// * Initialize storage.
///
/// * Start both acceptor and command handler runtimes.
///
/// * If server is started in slave mode, establish connection with master server, perform
/// a handshake and start listening to the replication stream from the master server. This happens inside
/// the acceptor tokio runtime.
fn main() -> Result<()> {
    env_logger::init();

    // parse CLI args
    let cli = Cli::parse();

    // Tokio runtime used for accepting TCP connections
    let acceptor_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .thread_name("acceptor-pool")
        .thread_stack_size(2 * 1024 * 1024)
        // .global_queue_interval(64)
        // .event_interval(200)
        // .max_io_events_per_tick(2048)
        .enable_all()
        .build()?;

    // Tokio runtime used for handling commands coming through a TCP stream
    let cmd_handler_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .thread_name("cmd-handler-pool")
        .thread_stack_size(2 * 1024 * 1024)
        .enable_all()
        .build()?;

    // Generate a 40 character alphanumeric replication id.
    // If server is started as a slave, try parsing the master host and port
    let replication_id = Alphanumeric.sample_string(&mut rand::thread_rng(), 40);
    let mut master_stream: Option<std::net::TcpStream> = None;
    let master_host_port = match cli.parse_master_host_port() {
        Ok(hp) => hp,
        Err(e) => panic!("{}", e),
    };

    // Try connecting to master server.
    // A panic can occur if the server fails to connect with the master server.
    if let Some((host, port)) = master_host_port.clone() {
        let master_addr = format!("{}:{}", host, port,);
        master_stream = match std::net::TcpStream::connect(master_addr.clone()) {
            Ok(stream) => {
                // set to non-blocking since its going to be used inside tokio runtime
                if let Err(e) = stream.set_nonblocking(true) {
                    error!("Failed to set master TCP stream to non-blocking: {}", e);
                    panic!("Failed to connect with master at {}", master_addr);
                }

                Some(stream)
            }
            Err(e) => {
                error!("{}", e);
                panic!("Failed to connect with master at {}", master_addr);
            }
        };
    }

    // Wrap the replication details into 2 separate Arcs (1 for each tokio runtimes).
    let replication = Replication::new(replication_id, master_host_port);
    let replication_acceptor_arc = Arc::new(replication);
    let replication_cmd_handler_arc = Arc::clone(&replication_acceptor_arc);

    // Initialize storage and wrap them into 2 separate Arcs (1 for each tokio runtimes)
    let shared_storage = storage::db::Storage::new(storage::db::DB::new());
    let storage_acceptor_arc = Arc::new(shared_storage);
    let storage_cmd_handler_arc = Arc::clone(&storage_acceptor_arc);

    // Channel for sending TcpStreams from acceptor runtime to command handler runtime
    let (tx, mut rx) = mpsc::channel::<TcpStream>(1000);

    // Spawn task for handling commands (command handler runtime)
    cmd_handler_runtime.spawn(async move {
        let mut server = Server::new(storage_cmd_handler_arc, replication_cmd_handler_arc);

        while let Some(stream) = rx.recv().await {
            server.handle_commands(stream).await
        }
    });

    // Run the acceptor runtime
    acceptor_runtime.block_on(async move {
        let port = cli.port.unwrap_or(DEFAULT_PORT);

        // If slave server, initialize master server listener
        if let Some(stream) = master_stream {
            let master_stream = match TcpStream::from_std(stream) {
                Ok(ms) => ms,
                Err(e) => {
                    error!("Error using master TCP stream inside async runtime: {}", e);
                    panic!("Failed to connect with master TCP stream: {}", e);
                },
            };

            let stream = match MasterServer::perform_handshake(master_stream).await {
                Ok(s) => s,
                Err(e) => panic!("Handshake with master server failed with error: {}", e),
            };

            tokio::spawn(async move {
                tokio::select! {
                    res = MasterServer::listen(stream, storage_acceptor_arc, replication_acceptor_arc) => {
                        if let Err(err) = res {
                            error!("failed to process the request from master: {}", err);
                        }
                    }
                }
            });
        }

        // Bind server to the specified port
        let addr = format!("127.0.0.1:{}", port);
        let listener = match TcpListener::bind(&addr).await {
            Ok(tcp_listener) => tcp_listener,
            Err(e) => panic!("Could not bind the TCP listener to {}. Err: {}", &addr, e),
        };

        info!("Started TCP listener on port {}", port);

        // Start accepting TCP connections
        loop {
            let sock = match accept_conn(&listener).await {
                Ok(stream) => stream,
                Err(e) => {
                    error!("{}", e);
                    panic!("Error accepting connection");
                }
            };

            let _ = tx.clone().send(sock).await;
        }
    });

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
