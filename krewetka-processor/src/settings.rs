use crate::actors::storage::clickhouse::ClickhouseSettings;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ProcessorSettings {
    pub kafka_topic: String,
    pub kafka_brokers: String,
    // pub clickhouse_user: String,
    // pub clickhouse_password: String,
    // pub clickhouse_host: String,
    // pub clickhouse_port: u16,
    pub clickhouse_settings: ClickhouseSettings,
    pub grpc_classification_port: u16,
    pub grpc_classification_host: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MigratorSettings {
    pub clickhouse_settings: ClickhouseSettings,
}
