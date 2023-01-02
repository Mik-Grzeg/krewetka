use super::errors::ExporterError;
use crate::application_state::HostIdentifier;
use async_trait::async_trait;
use log::{debug, info};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Receiver;
use tokio::task;
use tokio::time::Duration;

#[async_trait]
pub trait Export: Sync + Send {
    async fn export(&self, message: &[u8], identifier: &str) -> Result<(), ExporterError>;
}

pub async fn run(exporter: impl Export, rx: &mut Receiver<Vec<u8>>, identifier: &HostIdentifier) {
    info!("Spawned exporter...");
    let identifier = &String::from(identifier);

    let current = Arc::new(Mutex::new(0));
    let current_clone = current.clone();
    let timer = task::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;
            let mut current = current_clone.lock().unwrap();
            info!("Exporting events with a speed of: {} events per second", *current / 10);
            *current = 0;
        }
    });

    while let Some(m) = rx.recv().await {
        if exporter.export(&m, identifier).await.is_err() {
            debug!("Exporter is losing messages...");
        } else {
            let mut counter = current.lock().unwrap();
            *counter += 1;
        }
    }

    timer.await;
    info!("Closing exporter...");
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub Exporter {}

        #[async_trait]
        impl Export for Exporter {
            async fn export(&self, message: &[u8], identifier: &str) -> Result<(), ExporterError>;
        }
    }

    // #[test]
    // fn test_export_runner() {
    //     let mut exporter = MockExporter::new();
    //     let msg = b"test export runner";

    //     exporter.expect_export()
    //         .returning(move |_| Err(ExporterError::KafkaErr((KafkaError::Canceled, OwnedMessage::new(
    //             Some(msg.to_vec()),
    //             Some(b"KREWETKA".to_vec()),
    //             "test".to_string(),
    //             rdkafka::Timestamp::now(),
    //             32,
    //             200,
    //             None
    //         )))));

    //     let (tx, mut rx) = channel::<Vec<u8>>(10);
    //     task::spawn(async move {
    //         let event = "num: {}";
    //         for i in 1..15 as u8 {
    //             match tx.send(format!
    //                 ("{} {}", event, i).to_bytes().to_vec()
    //             ).await {
    //                 Ok(_) => debug!("num: {} .. OK", i),
    //                 Err(e) => error!("num: {} .. {}", i, e),
    //             };
    //         }
    //     });

    //     block_on(run(exporter, &mut rx))
    // }
}
