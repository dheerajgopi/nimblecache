use std::sync::Arc;

use anyhow::{anyhow, Result};
use bytes::BytesMut;
use log::{error, warn};

use crate::{commands::handler::RespCommandHandler, protocol::resp::types::RespType, server::info::{Master, ServerConfig}, storage::store::Store};

use super::handshake::Handshake;


const EMPTY_ARGS: Vec<RespType> = vec![];

pub struct MasterServer {

}

impl MasterServer {
    pub async fn start_listening(master: Master, store: Arc<Store>, svr_config: Arc<ServerConfig>) -> Result<()> {
        let mut master_stream = match Handshake::start(master).await {
            Ok(stream) => {stream},
            Err(e) => {return Err(anyhow!("Error while performing handshake. Error: {}", e))},
        };

        tokio::spawn(async move {
            loop {
                let buffer = BytesMut::with_capacity(512);
                let resp_handler = RespCommandHandler::from(
                    &mut master_stream,
                    buffer,
                    Arc::as_ref(&store),
                    svr_config.clone(),
                )
                .await;
                let (mut resp_handler, args) = match resp_handler {
                    Ok((h, args)) => (h, args),
                    Err(e) => {
                        warn!("Error reading the request: {}", e);
                        (RespCommandHandler::err_handler(&mut master_stream, e), EMPTY_ARGS)
                    }
                };

                // NULL handler is used to skip the loop in case 0 bytes are read from the stream
                match resp_handler {
                    RespCommandHandler::NULL => break,
                    _ => {}
                }

                let args = args.iter().map(|a| a).collect::<Vec<&RespType>>();
                let args = args.as_slice();
                match resp_handler.handle(args).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("error while handling RESP command: {}", e);
                    }
                };
            }
        });

        Ok(())
    }
}