use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

use super::errors::ExporterError;
use std::fmt::Debug;
use std::future::Future;

use log::{debug, error, info};

#[async_trait]
pub trait Export: Sync + Send {
    async fn export(&self, message: &[u8]);
}

pub async fn run(exporter: impl Export, rx: &mut Receiver<Vec<u8>>) {
    loop {
        // buffer = rx.recv();
        match rx.recv().await {
            Some(m) => exporter.export(&m).await,
            None => {
                error!("We've been tricked and quite possibly bamboozled. No message was found on the channel");
                return;
            }
        };
    }
}

// impl Debug for dyn Exporter {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         write!(f, "Exporter: {{{}}}", self.settings())
//     }
// }
