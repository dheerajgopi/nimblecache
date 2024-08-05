use anyhow::{Error, Result};
use futures::{SinkExt, StreamExt};
use log::error;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

use crate::resp::frame::RespCommandFrame;

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

            let mut resp_command_frame = Framed::new(sock, RespCommandFrame::new());

            tokio::spawn(async move {
                while let Some(resp_cmd) = resp_command_frame.next().await {
                    match resp_cmd {
                        Ok(cmd) => {
                            if let Err(e) = resp_command_frame.send(cmd).await {
                                error!("Error sending response: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Error reading the request: {}", e);
                            break;
                        }
                    };

                    match resp_command_frame.flush().await {
                        Ok(_) => {}
                        Err(_) => {}
                    };
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
