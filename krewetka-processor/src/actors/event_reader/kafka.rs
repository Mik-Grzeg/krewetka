use super::{Transport, EventStreamReaderActor};
use actix::Handler;
use actix::{Addr, Actor};

use std::error::Error;
use crate::pb::FlowMessage;
use super::FlowMessageWithMetadata;
use async_trait::async_trait;
use bytes::BytesMut;
use chrono::Utc;
use rdkafka::message::OwnedMessage;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tokio_stream::StreamMap;
use log::{debug, error, info, warn};
use prost::Message as PBMessage;
use super::super::messages::MessageFromEventStream;
use actix::Message as ActorMessage;
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer},
    error::KafkaResult,
    ClientConfig, ClientContext, Message, TopicPartitionList, message::BorrowedMessage,
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

    // pub fn get_stream(&self) -> impl Stream<Item = Result<FlowMessageWithMetadata, Box::<dyn Error>>> {
    //     self.consumer.stream()
    //         .map(|event| -> Result<FlowMessageWithMetadata, Box::<dyn Error>> {
    //             match event {
    //                 Ok(ev) => {
    //                     match ev.payload_view::<[u8]>() {
    //                         Some(Ok(f)) => {
    //                             let deserialized_msg: FlowMessage =
    //                                 PBMessage::decode::<&[u8]>(f).unwrap(); // TODO properly handle error

    //                             let msg_with_metadata: FlowMessageWithMetadata =
    //                                 FlowMessageWithMetadata {
    //                                     flow_message: deserialized_msg,
    //                                     timestamp: Utc::now(),
    //                                     host: "host".into(),
    //                                     malicious: None,
    //                                     // offset: ev.offset(),
    //                                 };

    //                             debug!(
    //                                 "Deserialized kafka event: {:?}",
    //                                 msg_with_metadata.flow_message
    //                             );
    //                             Ok(msg_with_metadata)
    //                         },
    //                         Some(Err(e)) => {
    //                             error!("Unable to decode kafka even into flow message: {:?}", e);
    //                             panic!("Shouldn't be here no message or error")
    //                         },
    //                         _ => panic!("Shouldn't be here no message or error")
    //                     }
    //                 }
    //                 Err(e) => {
    //                     error!("Unable to receive kafka event: {}", e);
    //                     panic!("Unable to receive kafka event");
    //                 },
    //             }
    //         })
    // }
}

#[async_trait]
impl Transport for KafkaState {
    async fn send_to_actor(&self, msg: OwnedMessage, next: &Addr<EventStreamReaderActor>) {
        match msg.payload_view::<[u8]>() {
            Some(Ok(f)) => {
                let deserialized_msg: FlowMessage =
                    PBMessage::decode::<&[u8]>(f).unwrap(); // TODO properly handle error

                let msg_with_metadata: FlowMessageWithMetadata =
                    FlowMessageWithMetadata {
                        flow_message: deserialized_msg,
                        timestamp: Utc::now(),
                        host: "host".into(),
                        malicious: None,
                        // offset: ev.offset(),
                    };

                debug!(
                    "Deserialized kafka event: {:?}",
                    msg_with_metadata.flow_message
                );
                let x = next.send(MessageFromEventStream { msg: msg_with_metadata}).await;
                info!("send reponse: {:?}", x);
            },
            Some(Err(e)) => {
                error!("Unable to decode kafka even into flow message: {:?}", e);
                panic!("Shouldn't be here no message or error")
            },
            _ => panic!("Shouldn't be here no message or error")
        }
    }

    async fn consume_batch(&self, tx: tokio::sync::broadcast::Sender<FlowMessageWithMetadata>) {
        info!("Started consuming kafka stream...");

        let mut streamer = self.consumer.stream();
        let mut i = 0;
        while let Some(event) = streamer.next().await {
            warn!("Iteration: {}", i); // TODO remove that debugging statement
            match event {
                Ok(ev) => {
                    match ev.payload_view::<[u8]>() {
                        Some(Ok(f)) => {
                            let deserialized_msg: FlowMessage =
                                PBMessage::decode::<&[u8]>(f).unwrap(); // TODO properly handle error

                            let msg_with_metadata: FlowMessageWithMetadata =
                                FlowMessageWithMetadata {
                                    flow_message: deserialized_msg,
                                    timestamp: Utc::now(),
                                    host: "host".into(),
                                    malicious: None,
                                };

                            debug!(
                                "Deserialized kafka event: {:?}",
                                msg_with_metadata.flow_message
                            );
                            tx.send(msg_with_metadata).unwrap()
                        }
                        Some(Err(e)) => {
                            error!("Unable to decode kafka even into flow message: {:?}", e);
                            continue;
                        }
                        _ => {
                            continue;
                        }
                    }
                }
                Err(e) => {
                    error!("Unable to receive kafka event: {}", e);
                    continue;
                }
            };
            i += 1; // TODO debugging purposes
        }
    }

    async fn consume(&self, next: Addr<EventStreamReaderActor>) {
        info!("Starting to consume messages from kafka...");
        let mut streamer = self.consumer.stream();
        let mut i = 0;

        while let Some(event) = streamer.next().await {
            warn!("Iteration: {}", i); // TODO remove that debugging statement
            match event {
                Ok(ev) => {
                    let response = self.send_to_actor(ev.detach(), &next).await;
                    debug!("Actor response: {:?}", response);
                },
                Err(e) => {
                    error!("Unable to receive kafka event: {}", e);
                    continue;
                }
            }
        }
    }
}
