use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use peer::ReplicaPeers;
use tokio::net::TcpStream;

use crate::resp::types::RespType;

pub mod master;
pub mod peer;

/// This struct stores the replication specific information.
#[derive(Debug, Clone)]
pub struct Replication {
    /// Unique id assigned to the server.
    pub id: String,
    /// Replication offset.
    pub offset: Arc<AtomicU64>,
    /// Master host. This is set only if the server is started as a slave.
    master_host: Option<String>,
    /// Master port. This is set only if the server is started as a slave.
    master_port: Option<u16>,
    /// Contains the list of slave replicas.
    replica_peers: ReplicaPeers,
}

impl Replication {
    pub fn new(id: String, master_host_port: Option<(String, u16)>) -> Replication {
        let (master_host, master_port) = match master_host_port {
            Some((h, p)) => (Some(h), Some(p)),
            None => (None, None),
        };
        Replication {
            id,
            offset: Arc::new(AtomicU64::new(0)),
            master_host,
            master_port,
            replica_peers: ReplicaPeers::new(),
        }
    }

    /// Server is considered as slave if a master host is assigned.
    pub fn is_slave(&self) -> bool {
        self.master_host.is_some()
    }

    /// Returns the replication info in `<key>:<value>` format.
    pub fn info_str(&self) -> String {
        let mut s = String::new();
        s.push_str("role:");

        if self.is_slave() {
            s.push_str("slave");
        } else {
            let offset = self.offset.load(Ordering::SeqCst);
            s.push_str("master\n");
            s.push_str(format!("master_replid:{}\n", self.id).as_str());
            s.push_str(format!("master_repl_offset:{}\n", offset).as_str());
        }

        s.to_string()
    }

    /// Increment the offset value with the number of bytes written to the replication stream.
    pub fn incr_offset(&self, incr_by: u64) {
        self.offset.fetch_add(incr_by, Ordering::SeqCst);
    }

    /// Add a new slave replica.
    pub async fn add_replica_peer(&self, stream: TcpStream) {
        self.replica_peers.add_peer(stream).await;
    }

    /// Send RESP data which is to be broadcast to all replicas.
    pub async fn write_to_replicas(&self, resp_data: RespType) -> usize {
        self.replica_peers.replicate(resp_data).await
    }
}
