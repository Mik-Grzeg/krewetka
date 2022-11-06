use crate::actors::broker::Broker;
use crate::actors::classification_client_grpc::client::Classifier;

use crate::actors::event_stream::kafka::retrier::Retrier;

use crate::actors::event_stream::{kafka::KafkaProcessingAgent, EventStreamActor};

use crate::actors::storage::storage_actor::StorageActor;

use tokio::sync::Mutex as TokioMtx;

use crate::actors::storage::clickhouse::ClickhouseState;
use crate::consts::DEFAULT_ENV_VAR_PREFIX;
use crate::pb::flow_message_classifier_client::FlowMessageClassifierClient;
use crate::settings::ProcessorSettings;
use actix::Actor;

use config::builder::DefaultState;
use config::{Config, ConfigBuilder, Environment};

use crate::actors::classification_client_grpc;

use serde::Deserialize;
use std::sync::Arc;

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
        // deserialize env config
        let _deserialized_config =
            get_config::<ProcessorSettings>(&self.config).expect("Getting config failed");

        // starting event stream actor
        let broker = Arc::new(TokioMtx::new(Broker));

        // init storage actor
        StorageActor::new(self.clickhouse_state.clone(), broker.clone()).start();

        // init classification actor
        let grpc_client =
            match FlowMessageClassifierClient::connect(self.classification_state.dsn()).await {
                Ok(c) => c,
                Err(e) => {
                    panic!("unable to connect with classification server: {:?}", e)
                }
            };

        classification_client_grpc::client::ClassificationActor {
            client: grpc_client,
        }
        .start();

        let processing_agent = Arc::new(KafkaProcessingAgent::new("flows", &self.brokers));
        let retrier = Arc::new(Retrier::new(self.brokers.clone()));

        let event_stream_actor = EventStreamActor::new(processing_agent, retrier, broker);

        event_stream_actor.start();
    }
}
