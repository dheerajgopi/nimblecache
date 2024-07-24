use std::{collections::HashMap, hash::Hash, net::SocketAddr};

use tokio::sync::mpsc::UnboundedSender;

use crate::protocol::resp::types::RespType;

#[derive(Debug, Clone, Eq)]
pub struct Peer {
    addr: SocketAddr,
    // is_ready: bool,
}

impl Peer {
    fn key(&self) -> &SocketAddr {
        &self.addr
    }
}

impl Hash for Peer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state)
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Peer {
        Peer { addr }
    }
}

#[derive(Debug, Clone)]
pub struct Replica {
    pub peers: HashMap<Peer, UnboundedSender<RespType>>,
}

impl Replica {
    pub fn new() -> Replica {
        Replica {
            peers: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, peer: Peer, sender: UnboundedSender<RespType>) {
        self.peers.insert(peer, sender);
    }
}
