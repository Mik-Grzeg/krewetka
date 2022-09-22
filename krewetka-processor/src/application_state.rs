use std::error::Error;

use crate::settings::Settings;
use crate::transport::{kafka, Transport};
use config::builder::DefaultState;
use config::{Config, ConfigBuilder, Environment};
use log::{debug, error};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{interval, Duration};
use crate::transport::kafka::{KafkaSettings, KafkaState};
use crate::storage::{astorage::StorageError, clickhouse::ClickhouseState};
use clickhouse_rs::Pool;

use crate::flow::FlowMessage;
use crate::transport::WrappedFlowMessage;

const DEFAULT_ENV_VAR_PREFIX: &str = "KREWETKA";

#[derive(Debug)]
pub enum ConfigErr {
    Read(config::ConfigError),
    MissingNeccessarySetting(String),
}

pub struct ApplicationState {
    config: Config,
    kafka_state: KafkaState,
    clickhouse_state: ClickhouseState,
}

impl ApplicationState {
    pub fn get_config<'d, T: Deserialize<'d>>(config: &Config) -> Result<T, ConfigErr> {
        config
            .clone()
            .try_deserialize()
            .map_err(ConfigErr::Read)
    }

    pub fn new() -> Result<Self, ConfigErr> {
        let base_config_builder = ConfigBuilder::<DefaultState>::default();
        let config = base_config_builder
            .add_source(Environment::with_prefix(DEFAULT_ENV_VAR_PREFIX).separator("__"))
            .build()
            .map_err(ConfigErr::Read)?;


        // deserialize env config
        let deserialized_config = Self::get_config::<Settings>(&config)
            .expect("Getting config failed");

        // set kafka settings
        let kafka_state = kafka::KafkaState::new(
            deserialized_config.kafka_topic,
            deserialized_config.kafka_brokers
        );
        // set clickhouse settings
        let clickhouse_state = ClickhouseState::new(
            deserialized_config.clickhouse_host,
            deserialized_config.clickhouse_port,
            deserialized_config.clickhouse_user,
            deserialized_config.clickhouse_password,
        );


        Ok(ApplicationState {
            config,
            kafka_state,
            clickhouse_state,
        })
    }

    pub async fn init(&self) {
        let (tx, mut rx) = mpsc::channel::<WrappedFlowMessage>(20);
        let mut ping_interval = interval(Duration::from_secs(15));

        let mut client = self.
            clickhouse_state
            .pool
            .as_ref()
            .get_handle()
            .await
            .map_err(|e| StorageError::Database(Box::new(e))).unwrap();

        task::spawn( async move {
            loop {
                if let Err(e) = client.ping().await {  error!("Ping does pong: {}", e)  };
                ping_interval.tick().await;
            }
        });

        task::spawn(async move {

            // while let Some(m) = rx.recv().await {
            //     debug!("Fetched: {:#?}", m.0);

            //     // TODO process and insert to clickhouse
            //     debug!("Inserting to clickhouse...")
            // }
        });

        self.kafka_state.consume_batch(tx).await;
    }
}
