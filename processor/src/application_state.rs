use crate::actors::classification_client_grpc::client::Classifier;
use crate::actors::event_stream::kafka::retrier::Retrier;
use crate::actors::event_stream::messages::{InitConsumer, InitRetrier};
use crate::actors::event_stream::{kafka::KafkaProcessingAgent, EventStreamActor};
use crate::actors::storage::messages::InitFlusher;
use crate::actors::storage::storage_actor::StorageActor;

use actix::clock::sleep;

use crate::actors::broker::Broker as MyBroker;

use crate::actors::storage::{clickhouse::ClickhouseState, storage_actor::StorageError};
use crate::consts::DEFAULT_ENV_VAR_PREFIX;
use crate::pb::flow_message_classifier_client::FlowMessageClassifierClient;
use crate::settings::ProcessorSettings;
use actix::Actor;
use actix_broker::Broker;
use actix_broker::SystemBroker;

use config::builder::DefaultState;
use config::{Config, ConfigBuilder, Environment};
use log::error;
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
        let actor_broker = Arc::new(Mutex::new(MyBroker));

        // init storage actor
        let storage_actor = StorageActor::new(self.clickhouse_state.clone(), actor_broker.clone());
        storage_actor.start();

        task::spawn(async move {
            Broker::<SystemBroker>::issue_async(InitFlusher {});
        })
        .await;

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

        // starting event stream actor
        let processing_agent = Arc::new(KafkaProcessingAgent::new("flows", &self.brokers));
        let retrier = Arc::new(Retrier::default());

        let event_stream_actor = EventStreamActor::<KafkaProcessingAgent, Retrier> {
            processor: processing_agent.clone(),
            retrier,
        };

        event_stream_actor.start();

        task::spawn(async move {
            Broker::<SystemBroker>::issue_async(InitRetrier);
        })
        .await;

        task::spawn(async move {
            Broker::<SystemBroker>::issue_async(InitConsumer);
        })
        .await;

        // TODO call healthcheck endpoint from here
        // for now it will be sleep
        sleep(Duration::from_secs(900)).await;
    }
}
