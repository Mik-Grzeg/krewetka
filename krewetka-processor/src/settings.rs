use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Settings {
    pub kafka_topic: String,
    pub kafka_brokers: String,
    pub clickhouse_user: String,
    pub clickhouse_password: String,
    pub clickhouse_host: String,
    pub clickhouse_port: u16,
}
