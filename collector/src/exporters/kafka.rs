use rdkafka::Message;
use std::fmt;

use std::time::Duration;

use chrono::Utc;
use uuid::Uuid;
use async_trait::async_trait;
use log::{debug, error};
use rdkafka::config::ClientConfig;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::{FutureProducer, FutureRecord};

use super::errors::ExporterError;
use super::exporter::Export;

#[derive(Debug, Clone)]
pub struct KafkaSettings {
    pub brokers: Vec<String>,
    pub topic: String,
}

impl KafkaSettings {
    pub fn get_brokers_kafka_format(&self) -> String {
        self.brokers.join(",")
    }
}

pub struct KafkaExporter {
    settings: KafkaSettings,
    producer: FutureProducer,
}

impl fmt::Debug for KafkaExporter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.settings)
    }
}
impl KafkaExporter {
    pub fn new(settings: KafkaSettings) -> Result<KafkaExporter, ExporterError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", settings.get_brokers_kafka_format())
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        Ok(KafkaExporter { settings, producer })
    }
}

#[async_trait]
impl Export for KafkaExporter {
    async fn export(&self, msg: &[u8], identifier: &str) -> Result<(), ExporterError> {
        // send event to kafka
        let result = self
            .producer
            .send(
                FutureRecord::to(&self.settings.topic)
                    .payload(msg)
                    .key("KREWETKA")
                    .headers(OwnedHeaders::new()
                            .add::<str>("host-identifier-x", identifier)
                            .add::<str>("message-id-x", &Uuid::new_v4().to_string())
                            .add::<str>("timestamp-x", &Utc::now().timestamp_millis().to_string())
                            .add::<str>("retry-x", &0.to_string())
                            // .add::<bool>("proto-encoding-x", true)
                    ),
                Duration::from_secs(0),
            )
            .await;

        // describe if it was successfull
        match result {
            Ok((partition, offset)) => {
                debug!(
                    "Event saved at partition: {}\toffset: {}",
                    partition, offset
                );
                Ok(())
            }
            Err((kafka_err, owned_msg)) => {
                error!(
                    "Unable to send message: {}\nPayload: {:?}",
                    kafka_err,
                    owned_msg.payload()
                );
                Err(ExporterError::from((kafka_err, owned_msg)))
            }
        }
    }
}