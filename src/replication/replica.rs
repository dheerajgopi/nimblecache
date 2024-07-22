use std::net::SocketAddr;
use tokio::net::TcpStream;

pub struct Replica {
    stream: TcpStream,
    addr: SocketAddr,
    is_ready: bool
}