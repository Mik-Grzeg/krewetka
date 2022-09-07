use std::fmt;
use std::future::Future;
use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::message::{Headers, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use log::error;



use super::publisher::{Export};
use super::errors::ExporterError;

#[derive(Debug, Clone)]
pub struct KafkaSettings {
    pub brokers: Vec<String>,
    pub topic: String,
}

impl KafkaSettings {
    pub fn builder() -> KafkaSettingsBuilder {
        KafkaSettingsBuilder::default()
    }

    pub fn get_brokers_kafka_format(&self) -> String {
        self.brokers.join(",")
    }

}



#[derive(Default)]
pub struct KafkaSettingsBuilder {
    brokers: Vec<String>,
    topic: Option<String>,
}

impl KafkaSettingsBuilder {
    pub fn new() -> KafkaSettingsBuilder {
        KafkaSettingsBuilder {
            brokers: Vec::new(),
            topic: None
        }
    }


    pub fn add_address(mut self, host: String, port: u16) -> KafkaSettingsBuilder {
        self.brokers.push(format!("{}:{}", host, port));
        self
    }

    pub fn topic(mut self, topic: String) -> KafkaSettingsBuilder {
        self.topic = Some(topic);
        self
    }

    pub fn build(self) -> KafkaSettings {
        KafkaSettings {
            brokers: self.brokers,
            topic: self.topic.expect("missing topic"),
        }
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

        Ok(KafkaExporter {
            settings: settings,
            producer: producer,
        })
    }

    pub async fn export(&self, message: Vec<u8>) -> Result<(i32, i64), ExporterError> {
        self.producer
            .send(
                FutureRecord::to(&self.settings.topic)
                .payload(&message)
                .key("KREWETKA")
                .headers(OwnedHeaders::new()
                    .add::<String>( "header_key", &"header_value".to_string()) 
                ),
                Duration::from_secs(0),
            ).await.map_err(|e| e.into())
    }
}

// impl Export for KafkaExporter {
//     fn

//     fn settings(&self) -> String {
//         format!("{}", self.settings())
//     }
// }