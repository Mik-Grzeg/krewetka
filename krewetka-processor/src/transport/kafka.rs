use std::sync::mpsc::Sender;

use super::Transport;

use async_trait::async_trait;
use log::{info, warn, error};
use rdkafka::{consumer::{BaseConsumer, Consumer, StreamConsumer, ConsumerContext, Rebalance, CommitMode}, ClientConfig, config::RDKafkaLogLevel, ClientContext, error::KafkaResult, TopicPartitionList, Message, message::Headers};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct KafkaSettings {
    pub brokers: Vec<String>,
    pub topic: String,
}

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        info!("Committing offsets: {:?}", result);
    }
}

pub struct KafkaState {
    consumer: StreamConsumer<CustomContext>,
    topic: String,
    brokers: String // comma separated brokers
}

impl KafkaState {
    pub fn new(topic: String, brokers: String) -> Self {
        let context = CustomContext;

        let consumer: StreamConsumer<CustomContext> = ClientConfig::new()
            .set("bootstrap.servers", brokers.clone())
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("group.id", "kafka-krewetka-group")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(context)
            .expect("Kafka consumer creation error");

        consumer.subscribe(&[&topic]);

        KafkaState {
            consumer,
            topic,
            brokers,
        }
    }
}

#[async_trait]
impl Transport for KafkaState {
    async fn consume(&self, tx: tokio::sync::mpsc::Sender<Vec<u8>>) {
        info!("Starting to consume messages from kafka...");
        loop {
            let msg = match self.consumer.recv().await {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Unable to consume kafka stream: {}", e);
                    break; 
                }
            };

            let payload = match msg.payload_view::<[u8]>() {
                Some(Ok(s)) => s,
                None => continue,
                Some(Err(e)) => {
                    warn!("Error while deserializing message payload: {:?}", e);
                    continue
                }
            };

            self.consumer.commit_message(&msg, CommitMode::Async).unwrap();
            tx.send(payload.to_owned()).await;  // TODO handle error
        }
    }
}
