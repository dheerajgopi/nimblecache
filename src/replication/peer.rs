use std::sync::Arc;

use log::{error, info};
use rand::distributions::{Alphanumeric, DistString};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::Mutex;

use crate::resp::types::RespType;

/// Stores a list of slave replicas and provides a mechanism to broadcast the replication stream to the peers.
/// This is maintained in master server.
#[derive(Debug, Clone)]
pub struct ReplicaPeers {
    /// The sender used to broadcast the replication stream to the peers.
    sender: Sender<RespType>,
    /// The list of connected peers.
    peers: Arc<Mutex<Vec<Peer>>>,
}

impl ReplicaPeers {
    /// Creates a new `ReplicaPeers` instance.
    pub fn new() -> ReplicaPeers {
        let (tx, _) = broadcast::channel(64);
        ReplicaPeers {
            sender: tx,
            peers: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Adds a new peer to the list of connected peers.
    ///
    /// # Arguments
    /// * `stream` - The `TcpStream` associated with the new peer.
    pub async fn add_peer(&self, stream: TcpStream) -> () {
        let rx = self.sender.subscribe();
        let peer_arc = self.peers.clone();
        let mut peers = peer_arc.lock().await;
        let new_peer = Peer::new(Arc::new(Mutex::new(rx)), Arc::new(Mutex::new(stream)));
        new_peer.init_replication(self.peers.clone()).await;
        peers.push(new_peer);

        info!("Number of peers connected: {}", peers.len());
    }

    /// Replicates the given `RespType` data to all connected peers.
    ///
    /// # Arguments
    /// * `resp_data` - The `RespType` data to be sent to the replication stream.
    pub async fn replicate(&self, resp_data: RespType) {
        let peers = self.peers.lock().await;

        if peers.len() == 0 {
            return;
        }

        if let Err(e) = self.sender.send(resp_data) {
            error!("{}", e);
        }
    }
}

/// Represents a single peer in the replication system.
#[derive(Debug, Clone)]
struct Peer {
    /// The unique identifier of the peer.
    id: String,
    /// The receiver to listen for replication updates.
    rx: Arc<Mutex<Receiver<RespType>>>,
    /// The `TcpStream` associated with the peer.
    stream: Arc<Mutex<TcpStream>>,
}

impl Peer {
    /// Assign a random alphanumeric id and create a new `Peer` instance.
    ///
    /// # Arguments
    /// * `rx` - The receiver to listen for replication updates.
    /// * `stream` - The `TcpStream` associated with the peer.
    pub fn new(rx: Arc<Mutex<Receiver<RespType>>>, stream: Arc<Mutex<TcpStream>>) -> Peer {
        let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
        Peer { id, rx, stream }
    }

    /// Initializes the replication process for an individual peer.
    /// The data coming through the receiver channel is sent to the peer's TCP stream until
    /// the slave gets disconnected, or if some unknown error occurs. In such cases, the individual
    /// peer is removed from the peer list.
    ///
    /// # Arguments
    /// * `peer_list` - The list of connected peers.
    pub async fn init_replication(&self, peer_list: Arc<Mutex<Vec<Peer>>>) {
        let rx = self.rx.clone();
        let stream = self.stream.clone();
        let id = self.id.clone();

        // send data from channel receiver to the peer's TCP stream.
        tokio::spawn(async move {
            let mut rx = rx.lock().await;
            let mut stream = stream.lock().await;

            while let Ok(resp_data) = rx.recv().await {
                if let Err(e) = stream.write(&resp_data.to_bytes()).await {
                    error!("Error writing to replica: {}", e);
                    break;
                }
            }

            // in case of disconnection/error remove the peer from the peer list.
            let mut peers = peer_list.lock().await;
            if let Some(idx) = peers.iter().position(|x| x.id == id) {
                peers.remove(idx);
            };
        });
    }
}
