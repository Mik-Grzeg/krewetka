use core::fmt;

use crate::exporters::{KafkaExporter, KafkaSettings}; // Exporter};
use crate::importers::{Import, ZMQSettings, ZMQ};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum ImporterVariants {
    #[serde(rename = "zmq")]
    ZMQ,
}

#[derive(Debug)]
pub enum ConstructorErr {
    ZMQErr,
    KafkaErr,
}

impl ImporterVariants {
    pub fn construct_importer(
        &self,
        settings: ImporterSettings,
    ) -> Result<impl Import, ConstructorErr> {
        match *self {
            Self::ZMQ => Ok(ZMQ::new(ZMQSettings {
                address: settings.zmq_address.ok_or(ConstructorErr::ZMQErr)?,
                queue_name: settings.zmq_queue_name.ok_or(ConstructorErr::ZMQErr)?,
            })),
        }
    }
}

impl From<ImporterVariants> for String {
    fn from(variant: ImporterVariants) -> Self {
        match variant {
            ImporterVariants::ZMQ => "zmq".to_string(),
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

impl From<ExporterVariants> for String {
    fn from(variant: ExporterVariants) -> Self {
        match variant {
            ExporterVariants::Kafka => "kafka".to_string(),
        }
    }
}

impl ExporterVariants {
    pub fn construct_exporter(
        &self,
        settings: ExporterSettings,
    ) -> Result<KafkaExporter, ConstructorErr> {
        match *self {
            Self::Kafka => Ok(KafkaExporter::new(KafkaSettings {
                brokers: settings
                    .kafka_brokers
                    .ok_or(ConstructorErr::KafkaErr)?
                    .split(",")
                    .map(|s| s.to_string())
                    .collect(),
                topic: settings.kafka_topic.ok_or(ConstructorErr::KafkaErr)?,
            })
            .expect("Wrong kafka config")),
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
    pub kafka_brokers: Option<String>,

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
    use crate::config::ConfigCache;
    use pretty_assertions::assert_eq;
    use serde_yaml;
    use serial_test::serial;
    use std::env;
    use test_case::test_case;

    fn parse_option_string(s: &Option<String>) -> String {
        match s {
            Some(s1) => s1.to_string(),
            None => "".to_string(),
        }
    }

    #[test_case(
        ImporterVariants::ZMQ,
        Some(String::from("localhost:5561")),
        Some(String::from("test"))
    )]
    #[test_case(ImporterVariants::ZMQ, None, None)]
    #[test_case(ImporterVariants::ZMQ, Some(String::from("localhost:5561")), None)]
    #[test_case(ImporterVariants::ZMQ, None, Some(String::from("test")))]
    fn test_importer_config_deserialization(
        source: ImporterVariants,
        address: Option<String>,
        queue_name: Option<String>,
    ) {
        let (exporter_yaml, exporter) = mock_exporter();
        let cfg = serde_yaml::from_str(&format!(
            "
        importer:
            source: {}
            settings:
              zmq_address: {}
              zmq_queue_name: {}
        {}
        ",
            source,
            parse_option_string(&address),
            parse_option_string(&queue_name),
            exporter_yaml
        ))
        .expect("unable to deserialize config");
        println!("{:?}", cfg);

        assert_eq!(
            Configuration {
                importer: Importer {
                    settings: ImporterSettings {
                        zmq_address: address,
                        zmq_queue_name: queue_name,
                    },
                    source: source,
                },
                exporter: exporter
            },
            cfg
        );
    }

    fn mock_exporter() -> (String, Exporter) {
        let yaml = "exporter:
          destination: kafka
          settings:
            kafka_brokers: localhost:9092,localhost:9091
            kafka_topic: test
        ";

        let obj = Exporter {
            destination: ExporterVariants::Kafka,
            settings: ExporterSettings {
                kafka_brokers: Some("localhost:9092,localhost:9091".to_string()),
                kafka_topic: Some("test".to_string()),
            },
        };

        (yaml.to_string(), obj)
    }

    #[test_case(ImporterVariants::ZMQ, Some(String::from("localhost:5561")), Some(String::from("test")), ExporterVariants::Kafka, Some(String::from("broker5:9092")), Some(String::from("test_topic")); "single broker")]
    #[test_case(ImporterVariants::ZMQ, Some(String::from("localhost:5561")), Some(String::from("test")), ExporterVariants::Kafka, Some(String::from("broker1:9092,broker2:9092")), Some(String::from("test_topic")); "two brokers")]
    #[test_case(ImporterVariants::ZMQ, Some(String::from("localhost:5561")), Some(String::from("test")), ExporterVariants::Kafka, Some(String::from("broker:9092,")), Some(String::from("test_topic")); "single broker with trailing comma")]
    #[test_case(ImporterVariants::ZMQ, Some(String::from("localhost:5561")), Some(String::from("test")), ExporterVariants::Kafka, Some(String::from("")), Some(String::from("test_topic")); "missing broker")]
    #[serial]
    fn test_env_configs(
        source: ImporterVariants,
        zmq_address: Option<String>,
        zmq_queue_name: Option<String>,
        destination: ExporterVariants,
        kafka_brokers: Option<String>,
        kafka_topic: Option<String>,
    ) {
        // create Settings object for importer and exporter with data provided in test cases
        let importer_settings = ImporterSettings {
            zmq_address: zmq_address.clone(),
            zmq_queue_name: zmq_queue_name.clone(),
        };
        let exporter_settings = ExporterSettings {
            kafka_brokers: kafka_brokers.clone(),
            kafka_topic: kafka_topic.clone(),
        };

        // expected configuration
        let configuration = Configuration {
            importer: Importer {
                source: source.clone(),
                settings: importer_settings,
            },
            exporter: Exporter {
                destination: destination.clone(),
                settings: exporter_settings,
            },
        };

        let env_setter = |key, val| {
            env::set_var(key, parse_option_string(val));
        };

        // set env vars with the data provided in test cases
        env_setter("KREWETKA__IMPORTER__SETTINGS__ZMQ_ADDRESS", &zmq_address);
        env_setter(
            "KREWETKA__IMPORTER__SETTINGS__ZMQ_QUEUE_NAME",
            &zmq_queue_name,
        );
        env_setter(
            "KREWETKA__EXPORTER__SETTINGS__KAFKA_BROKERS",
            &kafka_brokers,
        );
        env_setter("KREWETKA__EXPORTER__SETTINGS__KAFKA_TOPIC", &kafka_topic);

        env::set_var(
            "KREWETKA__EXPORTER__DESTINATION",
            &String::from(destination),
        );
        env::set_var("KREWETKA__IMPORTER__SOURCE", &String::from(source));

        // ConfigCache without configuration file
        let config_cache = ConfigCache::new("");

        let cfg = match config_cache {
            Ok(cfg) => cfg,
            Err(e) => {
                assert!(false, "{:?}", e);
                return;
            }
        };
        let config = match cfg.get_config::<Configuration>() {
            Ok(c) => c,
            Err(e) => {
                assert!(false, "{:?}", e);
                return;
            }
        };

        // ensure expected configuration is equal to the generated one
        assert_eq!(config, configuration);
    }
}
