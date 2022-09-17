use crate::settings::Settings;
use crate::transport::{kafka, Transport};
use config::builder::DefaultState;
use config::{Config, ConfigBuilder, Environment};
use log::{debug, warn};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::task;

const DEFAULT_ENV_VAR_PREFIX: &str = "KREWETKA";

#[derive(Debug)]
pub enum ConfigErr {
    Read(config::ConfigError),
}

pub struct ApplicationState {
    config: Config,
}

impl ApplicationState {
    pub fn get_config<'d, T: Deserialize<'d>>(&self) -> Result<T, ConfigErr> {
        Ok(self
            .config
            .clone()
            .try_deserialize()
            .map_err(ConfigErr::Read)?)
    }

    pub fn new() -> Result<Self, ConfigErr> {
        let base_config_builder = ConfigBuilder::<DefaultState>::default();
        let config = base_config_builder
            .add_source(Environment::with_prefix(&DEFAULT_ENV_VAR_PREFIX).separator("__"))
            .build()
            .map_err(ConfigErr::Read)?;

        Ok(ApplicationState { config })
    }

    pub async fn init(&self) {
        let config = self
            .get_config::<Settings>()
            .expect("Getting config failed");

        let topic = config.kafka_topic.unwrap();
        let brokers = config.kafka_brokers.unwrap();
        let kafka_state = kafka::KafkaState::new(topic, brokers);

        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(20);

        task::spawn(async move {
            kafka_state.consume(tx).await;
        });

        while let Some(m) = rx.recv().await {
            debug!("Fetched: {}", String::from_utf8_lossy(&m));

            // TODO process and insert to clickhouse
            debug!("Inserting to clickhouse...")
        }
    }
}
