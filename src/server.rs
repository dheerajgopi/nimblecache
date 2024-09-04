use std::sync::Arc;

use log::error;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::{
    handler::FrameHandler, replication::Replication, resp::frame::RespCommandFrame,
    storage::db::Storage,
};

/// Represents a TCP server that listens for and handles RESP commands.
#[derive(Debug)]
pub struct Server {
    /// Contains the storage.
    storage: Arc<Storage>,
    /// Contains the replication info.
    replication: Arc<Replication>,
}

impl Server {
    /// Creates a new `Server` instance.
    pub fn new(storage: Arc<Storage>, replication: Arc<Replication>) -> Server {
        Server {
            storage,
            replication,
        }
    }

    /// Reads the Nimblecache commands as tokio-util frames from the incoming TCP stream,
    /// and handle them in a separate Tokio async task.
    pub async fn handle_commands(&mut self, sock: TcpStream) {
        let db = self.storage.as_ref().db().clone();
        let replication = Arc::clone(&self.replication);
        let resp_command_frame = Framed::with_capacity(sock, RespCommandFrame::new(), 8 * 1024);

        tokio::spawn(async move {
            let handler = FrameHandler::new(resp_command_frame);
            if let Err(e) = handler.handle(db.as_ref(), replication.as_ref()).await {
                error!("Failed to handle command: {}", e);
            }
        });
    }
}
