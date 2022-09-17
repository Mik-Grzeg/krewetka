use log::{error, info};
use prost::EncodeError;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use super::errors::ImporterError;
use async_trait::async_trait;
use bytes::BufMut;
use prost::Message;

use crate::flow::FlowMessage;

#[async_trait]
pub trait Import: Sync + Send {
    async fn import(&self) -> Result<FlowMessage, ImporterError>;
}

pub async fn run(importer: impl Import, tx: Sender<Vec<u8>>) {
    info!("Spawned importer...");

    while let Ok(m) = importer.import().await {
        let mut buffer: Vec<u8> = Vec::with_capacity(4092);

        if let Err(e) = m.encode(&mut buffer) {
            error!("FlowMessage could not be encoded to bytes: {}", e);
            continue;
        }

        if let Err(e) = tx.send(buffer).await {
            error!(
                "unable to send fetched message to an exporter channel: {:?}",
                e
            );
            break;
        }
    }

    info!("Closing importer...");
}

pub trait Subscriber {
    fn recv(&self) -> Result<Vec<u8>, ImporterError>;
}
