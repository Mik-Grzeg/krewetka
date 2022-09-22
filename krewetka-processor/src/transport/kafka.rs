use std::iter::FilterMap;

use super::Transport;

use tokio_stream::{self as stream, StreamExt};
use crate::flow::FlowMessage;
use async_trait::async_trait;
use bytes::BytesMut;
use log::{debug, error, info, warn};
use prost::Message as PBMessage;
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer},
    error::KafkaResult,
    ClientConfig, ClientContext, Message, TopicPartitionList,
};

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
    brokers: String, // comma separated brokers
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


use rdkafka::message::FromBytes;
use prost::DecodeError;
use std::sync::{Arc, Mutex};
use super::utils::WrappedFlowMessage;

#[async_trait]
impl Transport for KafkaState {
    async fn consume_batch(&self, tx: tokio::sync::mpsc::Sender<WrappedFlowMessage>){
        info!("Started consuming kafka stream...");

        let mut streamer = self.consumer.stream();
        let mut i = 0;
        while let Some(event) = streamer.next().await {
            warn!("Iteration: {}", i); // TODO remove that debugging statement
            match event {
                Ok(ev) => { match ev.payload_view::<[u8]>(){
                    Some(Ok(f)) =>  {
                        let deserialized_msg = WrappedFlowMessage(
                            Arc::new(Mutex::new(PBMessage::decode::<&[u8]>(f).unwrap()))
                        ); // TODO properly handle error

                        debug!("Deserialized kafka event: {:?}", deserialized_msg.0);
                        tx.send(deserialized_msg).await
                    },
                    Some(Err(e)) => {
                        error!("Unable to decode kafka even into flow message: {:?}", e);
                        continue;
                    },
                    _ => { continue; },
                }},
                Err(e) => {
                    error!("Unable to receive kafka event: {}", e);
                    continue;
                }
            };
            i+=1; // TODO debugging purposes
        }
    }

    async fn consume(&self, tx: tokio::sync::mpsc::Sender<FlowMessage>) {
        info!("Starting to consume messages from kafka...");
        let _buf = BytesMut::with_capacity(4096);
        loop {
            let msg = match self.consumer.recv().await {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Unable to consume kafka stream: {}", e);
                    break;
                }
            };
//
            let payload = match msg.payload_view::<[u8]>() {
                Some(Ok(s)) => s,
                None => continue,
                Some(Err(e)) => {
                    warn!("Error while deserializing message payload: {:?}", e);
                    continue;
                }
            };
//
            // handling commiting error
            if let Err(e) = self.consumer.commit_message(&msg, CommitMode::Async) {
                error!("Unable to commit message in kafka: {}", e)
            }
//
            let decoded_msg: FlowMessage = match PBMessage::decode::<&[u8]>(payload) {
                Ok(m) => {
                    debug!("Received event: {:?}", m);
                    m
                }
                Err(e) => {
                    error!("Unable to decode event: {}", e);
                    continue;
                }
            };
//
            // handling channel sending error
            if let Err(e) = tx.send(decoded_msg).await {
                error!("Unable to put kafka event on channel: {}", e)
            }
        }
    }
}
