use super::{EventStreamReaderActor, Transport};

use crate::actors::messages::FlowMessageWithMetadata;
use actix::Addr;
use actix_broker::{Broker, SystemBroker};

use crate::pb::FlowMessage;

use async_trait::async_trait;

use chrono::Utc;
use log::{debug, error, info};
use prost::Message as PBMessage;
use rdkafka::message::OwnedMessage;
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{Consumer, ConsumerContext, Rebalance, StreamConsumer},
    error::KafkaResult,
    ClientConfig, ClientContext, Message, TopicPartitionList,
};

use tokio_stream::StreamExt;

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
    async fn send_to_actor(&self, msg: OwnedMessage, _next: &Addr<EventStreamReaderActor>) {
        match msg.payload_view::<[u8]>() {
            Some(Ok(f)) => {
                let deserialized_msg: FlowMessage = PBMessage::decode::<&[u8]>(f).unwrap(); // TODO properly handle error

                let msg_with_metadata: FlowMessageWithMetadata = FlowMessageWithMetadata {
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
                Broker::<SystemBroker>::issue_async::<FlowMessageWithMetadata>(msg_with_metadata);
            }
            Some(Err(e)) => {
                error!("Unable to decode kafka even into flow message: {:?}", e);
                panic!("Shouldn't be here no message or error")
            }
            _ => panic!("Shouldn't be here no message or error"),
        }
    }

    async fn consume_batch(&self, tx: tokio::sync::broadcast::Sender<FlowMessageWithMetadata>) {
        info!("Started consuming kafka stream...");

        let mut streamer = self.consumer.stream();
        while let Some(event) = streamer.next().await {
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
        }
    }

    async fn consume(&self, next: Addr<EventStreamReaderActor>) {
        info!("Starting to consume messages from kafka...");
        let mut streamer = self.consumer.stream();

        while let Some(event) = streamer.next().await {
            match event {
                Ok(ev) => {
                    let response = self.send_to_actor(ev.detach(), &next).await;
                    debug!("Actor response: {:?}", response);
                }
                Err(e) => {
                    error!("Unable to receive kafka event: {}", e);
                    continue;
                }
            }
        }
    }
}
