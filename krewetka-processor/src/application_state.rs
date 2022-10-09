use crate::classification_client::{streaming_classifier, Classifier};
use crate::consts::DEFAULT_ENV_VAR_PREFIX;
use crate::pb::flow_message_classifier_client::FlowMessageClassifierClient;
use crate::settings::ProcessorSettings;
use crate::storage::astorage::AStorage;
use crate::storage::{astorage::StorageError, clickhouse::ClickhouseState};
use crate::transport::kafka::KafkaState;
use crate::transport::{kafka, FlowMessageWithMetadata, Transport};
use config::builder::DefaultState;
use config::{Config, ConfigBuilder, Environment};
use log::error;

use log::debug;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio::task;
use tokio::time::{interval, Duration};

// use crate::transport::WrappedFlowMessage;

#[derive(Debug)]
pub enum ConfigErr {
    Read(config::ConfigError),
    MissingNeccessarySetting(String),
}

pub struct ApplicationState {
    config: Config,
    kafka_state: Arc<KafkaState>,
    clickhouse_state: Arc<ClickhouseState>,
    classification_state: Classifier,
}

pub fn get_config<'d, T: Deserialize<'d>>(config: &Config) -> Result<T, ConfigErr> {
    config.clone().try_deserialize().map_err(ConfigErr::Read)
}

impl ApplicationState {
    pub async fn new() -> Result<Self, ConfigErr> {
        let base_config_builder = ConfigBuilder::<DefaultState>::default();
        let config = base_config_builder
            .add_source(Environment::with_prefix(DEFAULT_ENV_VAR_PREFIX).separator("__"))
            .build()
            .map_err(ConfigErr::Read)?;

        // deserialize env config
        let deserialized_config =
            get_config::<ProcessorSettings>(&config).expect("Getting config failed");

        // set kafka settings
        let kafka_state = Arc::new(kafka::KafkaState::new(
            deserialized_config.kafka_topic,
            deserialized_config.kafka_brokers,
        ));
        // set clickhouse settings
        let clickhouse_state = Arc::new(ClickhouseState::new(
            deserialized_config.clickhouse_settings,
        ));

        let classification_state = Classifier {
            port: deserialized_config.grpc_classification_port,
            host: deserialized_config.grpc_classification_host,
        };

        Ok(ApplicationState {
            config,
            kafka_state,
            clickhouse_state,
            classification_state,
        })
    }

    pub async fn init(&self) {
        let (tx, mut rx) = broadcast::channel::<FlowMessageWithMetadata>(100);
        let (tx_after_ml, mut rx_after_ml) = mpsc::channel::<FlowMessageWithMetadata>(100);
        let mut ping_interval = interval(Duration::from_secs(15));

        let mut client = self
            .clickhouse_state
            .pool
            .as_ref()
            .get_handle()
            .await
            .map_err(|e| StorageError::Database(Box::new(e)))
            .unwrap();

        task::spawn(async move {
            loop {
                if let Err(e) = client.ping().await {
                    error!("Ping does pong: {}", e)
                };
                ping_interval.tick().await;
            }
        });

        let rx_1 = tx.subscribe();
        let stream_channel = tokio_stream::wrappers::BroadcastStream::from(rx_1);

        // create grpc ml client
        let mut grpc_client = FlowMessageClassifierClient::connect(format!(
            "http://{}:{}",
            self.classification_state.host, self.classification_state.port
        ))
        .await
        .unwrap();
        let ml_pipeline = task::spawn(async move {
            streaming_classifier(&mut rx, stream_channel, tx_after_ml, &mut grpc_client).await;
        });

        let clickhouse_state = self.clickhouse_state.clone();
        let mut msgs: Vec<FlowMessageWithMetadata> = Vec::with_capacity(100);

        let processing = task::spawn(async move {
            while let Some(m) = rx_after_ml.recv().await {
                debug!("Fetched: {:#?}", m);
                msgs.push(m);

                if msgs.len() == 100 {
                    clickhouse_state.stash(&msgs).await.unwrap();
                    msgs.drain(..);
                }
            }
        });

        self.kafka_state.consume_batch(tx).await;
        ml_pipeline.await;
        processing.await;
    }
}
