use rdkafka::Message;
use std::fmt;

use async_trait::async_trait;
use chrono::Utc;

use log::error;
use rdkafka::config::ClientConfig;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::{BaseRecord, DefaultProducerContext, ThreadedProducer};
use uuid::Uuid;

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
    producer: ThreadedProducer<DefaultProducerContext>,
}

impl fmt::Debug for KafkaExporter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.settings)
    }
}
impl KafkaExporter {
    pub fn new(settings: KafkaSettings) -> Result<KafkaExporter, ExporterError> {
        let producer: ThreadedProducer<DefaultProducerContext> = ClientConfig::new()
            .set("bootstrap.servers", settings.get_brokers_kafka_format())
            .set("message.timeout.ms", "5000")
            // .set("queue.buffering.max.ms", "10")
            // .set("queue.buffering.max.messages", "1000")
            .create()
            .expect("Producer creation error");

        Ok(KafkaExporter { settings, producer })
    }
}

#[async_trait]
impl Export for KafkaExporter {
    async fn export(&self, msg: &[u8], identifier: &str) -> Result<(), ExporterError> {
        // send event to kafka
        let key = format!("KREWETKA-{}", &Uuid::new_v4().to_string());
        let record = BaseRecord::to(&self.settings.topic)
            .payload(msg)
            .key(&key)
            .headers(
                OwnedHeaders::new()
                    .add::<str>("host-identifier-x", identifier)
                    .add::<str>("message-id-x", &Uuid::new_v4().to_string())
                    .add::<str>("timestamp-x", &Utc::now().timestamp_millis().to_string())
                    .add::<str>("retry-x", &0.to_string()), // .add::<bool>("proto-encoding-x", true)
            );

        self.producer.send(record).map_err(|(e, record)| {
            error!("Unable to send message: {}\nPayload: {:?}", e, record);
            ExporterError::from(e)
        })
    }
}
