use crate::actors::classification_client_grpc::client::Classifier;
use crate::actors::event_stream::kafka::OffsetGuard;
use crate::actors::event_stream::TopicOffsetKeeper;
use crate::actors::storage::storage_actor::StorageActor;

use crate::actors::broker::Broker;
use crate::actors::event_stream::{
    kafka::{self, KafkaState},
    Transport,
};
use crate::actors::storage::storage_actor;
use crate::actors::storage::{clickhouse::ClickhouseState, storage_actor::StorageError};
use crate::consts::DEFAULT_ENV_VAR_PREFIX;
use crate::pb::flow_message_classifier_client::FlowMessageClassifierClient;
use crate::settings::ProcessorSettings;
use actix::Actor;

use config::builder::DefaultState;
use config::{Config, ConfigBuilder, Environment};
use log::error;
use std::sync::Mutex;

use crate::actors::{classification_client_grpc, event_stream};

use serde::Deserialize;
use std::sync::Arc;

use tokio::task;
use tokio::time::{interval, Duration};

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

        let state = ApplicationState {
            config,
            kafka_state,
            clickhouse_state,
            classification_state,
        };

        Ok(state)
    }

    pub async fn init_actors(&self) {
        // Clickhouse health check
        let mut ping_interval = interval(Duration::from_secs(15));

        // deserialize env config
        let _deserialized_config =
            get_config::<ProcessorSettings>(&self.config).expect("Getting config failed");

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

        // create wrapped actix broker
        let actor_broker = Arc::new(Mutex::new(Broker));

        // init storage actor
        let (storage_actor, recv) = StorageActor::new(self.clickhouse_state.clone());
        storage_actor.start();

        // spawn buffer flushing task
        let flusher_storage_state = self.clickhouse_state.clone();
        let flusher =
            task::spawn(
                async move { storage_actor::flush_buffer(flusher_storage_state, recv).await },
            );

        // init classification actor
        classification_client_grpc::client::ClassificationActor {
            client: FlowMessageClassifierClient::connect(format!(
                "http://{}:{}",
                self.classification_state.host, self.classification_state.port
            ))
            .await
            .unwrap(),
            broker: actor_broker.clone(),
        }
        .start();

        // start event streaming actor
        event_stream::EventStreamActor {
            channel: self.kafka_state.clone(),
        }
        .start();

        // start task which gruards offset commits
        let mut offset_guard = OffsetGuard::new(self.kafka_state.clone(), &self.kafka_state.topic);
        let acker = task::spawn(async move { offset_guard.inc_offset().await });

        // start consuming messages on kafka
        self.kafka_state.consume().await;

        flusher.await;
        acker.await;
    }
}
