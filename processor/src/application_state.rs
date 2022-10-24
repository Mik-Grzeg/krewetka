use crate::actors::classification_client_grpc::client::Classifier;
use crate::actors::event_stream::kafka::{ConsumerOffsetGuard, ProcessingAgent};
use crate::actors::event_stream::{ArcedRetrier, EventStreamActor, Retrier, TopicOffsetKeeper};
use crate::actors::storage::storage_actor::StorageActor;

use crate::actors::broker::Broker;
use crate::actors::event_stream::{kafka::KafkaState, Transport};
use crate::actors::storage::storage_actor;
use crate::actors::storage::{clickhouse::ClickhouseState, storage_actor::StorageError};
use crate::consts::DEFAULT_ENV_VAR_PREFIX;
use crate::pb::flow_message_classifier_client::FlowMessageClassifierClient;
use crate::settings::ProcessorSettings;
use actix::Actor;

use rdkafka::consumer::{StreamConsumer, Consumer};
use config::builder::DefaultState;
use config::{Config, ConfigBuilder, Environment};
use log::error;
use std::collections::BinaryHeap;
use std::sync::Mutex;

use crate::actors::classification_client_grpc;

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
    // kafka_settings: KafkaSettings,
    brokers: String,
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
        let brokers = deserialized_config.kafka_brokers;
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
            brokers,
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

        let consumer = Arc::new(KafkaState::get_consumer(&self.brokers));
        consumer
            .subscribe(&["flows"])
            .unwrap_or_else(|_| panic!("Unable to subscribe to topic {}", "flows"));

        let heap = BinaryHeap::new();
        let heap_ref = Arc::new(Mutex::new(heap));
        let mut offset_guard = ConsumerOffsetGuard::new(consumer.clone(), "flows", heap_ref.clone());

        // processing agent
        let producer = KafkaState::get_producer(&self.brokers);
        let processing_agent = Arc::new(ProcessingAgent::new(producer));

        let retrier = Arc::new(Retrier::default());

        let event_stream_actor = EventStreamActor {
            processor: processing_agent.clone(),
            retrier: retrier.clone(),
            offset_heap: heap_ref.clone(),
        };

        event_stream_actor.start();

        // start background retrier
        let wrapped_retrier = ArcedRetrier::new(retrier.clone());

        // // start task which gruards offset commits
        let acker = task::spawn(async move { offset_guard.inc_offset("flows".to_owned()).await });

        // retrier
        let retrier = wrapped_retrier.init_retry();

        // start consuming messages on kafka
        let processing_agent_clone = processing_agent.clone();
        let processing = processing_agent_clone.consume(consumer.clone(), "flows".to_owned(), self.brokers.clone());

        tokio::join!(retrier, processing, acker, flusher,);
    }
}
