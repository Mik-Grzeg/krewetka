use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;
use log::{error, info, debug};
use super::errors::ExporterError;

#[async_trait]
pub trait Export: Sync + Send {
    async fn export(&self, message: &[u8]) -> Result<(), ExporterError>;
}

pub async fn run(exporter: impl Export, rx: &mut Receiver<Vec<u8>>) {
    info!("Spawned exporter...");

    while let Some(m) = rx.recv().await  {
        if let Err(_) = exporter.export(&m).await {
            debug!("Exporter is losing messages...");
        }
    };

    info!("Closing exporter...");
}


#[cfg(test)]
mod tests {
    use super::*;
    use rdkafka::message::{OwnedMessage, ToBytes};
    use tokio_test::block_on;
    use pretty_assertions;
    use mockall::mock;
    use rdkafka::error::KafkaError;
    use tokio::sync::mpsc::channel;
    use tokio::task;
    
    mock!{
        pub Exporter {}

        #[async_trait]
        impl Export for Exporter {
            async fn export(&self, message: &[u8]) -> Result<(), ExporterError>;
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