use core::fmt;

use crate::importers::{ZMQ, ZMQSettings};
use crate::exporters::{KafkaExporter, KafkaSettings};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum ImporterVariants {
    #[serde(rename = "zmq")]
    ZMQ,
}

#[derive(Debug)]
pub enum ConstructorErr{
    ZMQErr,
    KafkaErr
}

impl ImporterVariants {
    pub fn construct_importer(&self, settings: ImporterSettings) -> Result<ZMQ, ConstructorErr> {
        match *self {
            Self::ZMQ => Ok(ZMQ::new(
                ZMQSettings {
                    address: settings.zmq_address.ok_or(ConstructorErr::ZMQErr)?,
                    queue_name: settings.zmq_queue_name.ok_or(ConstructorErr::ZMQErr)?,
                }
            ))
        }
    }
}

impl fmt::Display for ImporterVariants {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Self::ZMQ => "zmq",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Importer {
    pub source: ImporterVariants,
    pub settings: ImporterSettings,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ImporterSettings {
    pub zmq_address: Option<String>,

    pub zmq_queue_name: Option<String>,
}


#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum ExporterVariants {
    #[serde(rename = "kafka")]
    Kafka,
}

impl ExporterVariants {
    pub fn construct_exporter(&self, settings: ExporterSettings) -> Result<KafkaExporter, ConstructorErr> {
        match *self {
            Self::Kafka => Ok(KafkaExporter::new(
                KafkaSettings {
                    brokers: settings.kafka_brokers.ok_or(ConstructorErr::KafkaErr)?,
                    topic: settings.kafka_topic.ok_or(ConstructorErr::KafkaErr)?,
                }
            ).expect("Wrong kafka config"))
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Exporter {
    pub destination: ExporterVariants,
    pub settings: ExporterSettings,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExporterSettings {
    pub kafka_brokers: Option<Vec<String>>,

    pub kafka_topic: Option<String>,
}


#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Configuration {
    pub importer: Importer,

    pub exporter: Exporter,

}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    fn parse_option_string(s: &Option<String>) -> String {
        match s {
            Some(s1) => s1.to_string(),
            None => "".to_string()
        }
    }

    #[test_case(ImporterVariants::ZMQ, Some("localhost:5561".to_string()), Some("test_queue".to_string()))]
    #[test_case(ImporterVariants::ZMQ, None, None)]
    fn test_importer_config_deserialization(source: ImporterVariants, address: Option<String>, queue_name: Option<String>) {
        let (exporter_yaml, exporter) = mock_exporter();
        let cfg = serde_yaml::from_str(&format!("
        importer:
            source: {}
            settings:
              zmq_address: {}
              zmq_queue_name: {}
        {}
        ", source, parse_option_string(&address), parse_option_string(&queue_name), exporter_yaml)).expect("unable to deserialize config");
        println!("{:?}", cfg);


        assert_eq!(Configuration {
            importer: Importer {
                settings: ImporterSettings {
                    zmq_address: address,
                    zmq_queue_name: queue_name,
                },
                source: source, 
            },
            exporter: exporter 
        }, cfg);

    }

    fn mock_exporter() -> (String, Exporter) {
        let yaml = 
        "exporter: 
          destination: kafka
          settings:
            kafka_brokers: 
            - localhost:9092
            - localhost:9091
            kafka_topic: test
        ";

        let obj = Exporter { 
            destination: ExporterVariants::Kafka, 
            settings: ExporterSettings {
                kafka_brokers: Some(vec!("localhost:9092".to_string(), "localhost:9091".to_string())),
                kafka_topic: Some("test".to_string()),
            }
        };

        (yaml.to_string(), obj)
    }
}