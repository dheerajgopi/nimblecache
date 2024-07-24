mod cli;
mod commands;
mod protocol;
mod replication;
mod server;
mod storage;

use std::sync::Arc;

use crate::cli::args::Args;
use crate::server::srv::TcpServer;
use clap::Parser;
use env_logger;
use server::info::ServerConfig;

#[tokio::main]
async fn main() {
    let cli_args = Args::parse();

    // init logger
    env_logger::init();

    let srv_cfg = ServerConfig::new(&cli_args);
    let srv_cfg = match srv_cfg {
        Ok(si) => si,
        Err(e) => {
            panic!("Error while initializing server {}", e)
        }
    };
    let srv_cfg_arc = Arc::new(srv_cfg);

    // init server and start listening to the specified port
    let srv = TcpServer::new(&cli_args);
    srv.start(srv_cfg_arc).await;
}
