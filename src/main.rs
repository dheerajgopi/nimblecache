mod cli;
mod commands;
mod protocol;
mod replication;
mod server;
mod storage;

use crate::cli::args::Args;
use crate::server::srv::TcpServer;
use clap::Parser;
use env_logger;

#[tokio::main]
async fn main() {
    let cli_args = Args::parse();

    // init logger
    env_logger::init();

    // init server and start listening to the specified port
    let srv = TcpServer::new(&cli_args);
    srv.start().await;
}
