use anyhow::{Error, Result};
use log::error;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(listener: TcpListener) -> Server {
        Server { listener }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let mut sock = match self.accept_conn().await {
                Ok(stream) => stream,
                Err(e) => {
                    error!("{}", e);
                    panic!("Error accepting connection");
                }
            };

            tokio::spawn(async move {
                if let Err(e) = &mut sock.write_all("Hello!".as_bytes()).await {
                    error!("{}", e);
                    panic!("Error writing response")
                }
            });
        }
    }

    async fn accept_conn(&mut self) -> Result<TcpStream> {
        loop {
            match self.listener.accept().await {
                Ok((sock, _)) => return Ok(sock),
                Err(e) => return Err(Error::from(e)),
            }
        }
    }
}
