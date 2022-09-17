use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Settings {
    pub kafka_topic: Option<String>,
    pub kafka_brokers: Option<String>,
}
