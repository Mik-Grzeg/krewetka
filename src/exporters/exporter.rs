use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

use super::errors::ExporterError;
use std::fmt::Debug;
use std::future::Future;

#[async_trait]
pub trait Export: Sync + Send {
    async fn export(&self, message: &mut Receiver<Vec<u8>>);
}

// pub fn run(exporter: impl Export, rx: &mut Receiver<Vec<u8>>) {
//     exporter.export(message)
// }

// impl Debug for dyn Exporter {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         write!(f, "Exporter: {{{}}}", self.settings())
//     }
// }
