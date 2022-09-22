use serde::Deserialize;
use crate::storage::clickhouse::ClickhouseSettings;


#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ProcessorSettings {
    pub kafka_topic: String,
    pub kafka_brokers: String,
    // pub clickhouse_user: String,
    // pub clickhouse_password: String,
    // pub clickhouse_host: String,
    // pub clickhouse_port: u16,
    pub clickhouse_settings: ClickhouseSettings,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct MigratorSettings {
    pub clickhouse_settings: ClickhouseSettings,
}
