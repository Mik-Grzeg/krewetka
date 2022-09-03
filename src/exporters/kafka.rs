use kafka::producer::{Producer, Record};
use log::error;

use super::publisher::{Export, ExporterError};

#[derive(Debug, Clone)]
pub struct KafkaSettings {
    addresses: Vec<String>,
    topic: Option<String>,
}

impl KafkaSettings {
    pub fn builder() -> KafkaSettingsBuilder {
        KafkaSettingsBuilder::default()
    }
}



#[derive(Default)]
pub struct KafkaSettingsBuilder {
    addresses: Vec<String>,
    topic: Option<String>,
}

impl KafkaSettingsBuilder {
    pub fn new() -> KafkaSettingsBuilder {
        KafkaSettingsBuilder {
            addresses: Vec::new(),
            topic: None
        }
    }

    pub fn add_address(mut self, host: String, port: u16) -> KafkaSettingsBuilder {
        self.addresses.push(format!("{}:{}", host, port));
        self
    }

    pub fn topic(mut self, topic: String) -> KafkaSettingsBuilder {
        self.topic = Some(topic);
        self
    }

    pub fn build(self) -> KafkaSettings {
        KafkaSettings {
            addresses: self.addresses,
            topic: self.topic,
        }
    }
}


pub struct KafkaExporter {
    settings: KafkaSettings,
    client: Producer,
}

impl KafkaExporter {
    pub fn new(settings: KafkaSettings) -> Result<KafkaExporter, kafka::Error> {
        let client = match Producer::from_hosts(settings.addresses.clone()).create() {
                Ok(p) => p,
                Err(e) =>  {
                    error!("Unable to create producer using given settings");
                    return Err(e)
                }
            };

        Ok(KafkaExporter {
            settings: settings,
            client: client,
        })
    }
}

impl Export for KafkaExporter {
    fn export(&self) -> Result<(), ExporterError> {
        //TODO kafka exporter
        Ok(())
    }
}