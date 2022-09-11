use tokio::sync::mpsc::Sender;
use log::{error, info};
use std::sync::Arc;

use super::errors::ImporterError;
use async_trait::async_trait;


#[async_trait]
pub trait Import: Sync + Send {
    async fn import(&self) -> Result<Vec<u8>, ImporterError>;
}

pub async fn run(importer: impl Import, tx: Sender<Vec<u8>>) 
{
    info!("Spawned importer...");

    while let Ok(m) = importer.import().await {
        if let Err(e) = tx.send(m.to_vec()).await {
            error!(
                "unable to send fetched message to an exporter channel: {:?}",
                e
            );
            break;
        }
    };

    info!("Closing importer...");
}

pub trait Subscriber {
    fn recv(&self) -> Result<Vec<u8>, ImporterError>;
}