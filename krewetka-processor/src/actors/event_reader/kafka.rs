use super::{Transport};

use super::consts::OFFSET_COMMIT_INTERVAL;
use crate::actors::event_reader::topic::TopicOffsetKeeper;
use crate::actors::messages::FlowMessageWithMetadata;

use actix_broker::{Broker, SystemBroker};
use log::warn;
use rdkafka::error::KafkaError;
use std::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::actors::acker::Acknowleger;
use crate::pb::FlowMessage;
use std::collections::BinaryHeap;
use std::sync::{Arc};

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

pub struct OffsetGuard {
    processed_offests: Arc<Mutex<BinaryHeap<i64>>>,
    last_stored_offest: i64,
    state: Arc<KafkaState>,
}

impl OffsetGuard {
    pub fn new(state: Arc<KafkaState>, topic: &str) -> Self {
        let processed_offests = Arc::new(Mutex::new(BinaryHeap::new()));
        let last_stored_offest = OffsetGuard::get_offset_for_topic(&state.consumer, topic);

        Self {
            processed_offests,
            last_stored_offest,
            state: state.clone(),
        }
    }

    fn store_offset(&self, offset: i64) -> Result<(), KafkaError> {
        self.state
            .consumer
            .store_offset(&self.state.topic, 0, offset)
    }

    fn get_offset_for_topic(consumer: &StreamConsumer<CustomContext>, topic: &str) -> i64 {
        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_topic_unassigned(topic);

        match consumer.committed_offsets(tpl, Duration::from_secs(3)) {
            Ok(list) => {
                let elemts = list.elements_for_topic(topic);
                if elemts.len() == 1 {
                    return elemts[0].offset().to_raw().unwrap();
                } else {
                    panic!("Topic is missing: {}", topic);
                }
            }
            Err(err) => panic!("Kafka error: {}", err),
        };
    }
}

pub struct KafkaState {
    consumer: StreamConsumer<CustomContext>,
    pub topic: String,
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
            .set("enable.auto.offset.store", "false")
            .set("group.id", "kafka-krewetka-group")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(context)
            .expect("Kafka consumer creation error");

        if let Err(e) = consumer.subscribe(&[&topic]) {
            error!("Error while subscribing to kafka topic: {}", e)
        }

        KafkaState {
            consumer,
            topic,
            brokers,
        }
    }

    async fn send_to_actor(&self, msg: OwnedMessage) {
        match msg.payload_view::<[u8]>() {
            Some(Ok(f)) => {
                let deserialized_msg: FlowMessage = PBMessage::decode::<&[u8]>(f).unwrap(); // TODO properly handle error

                let msg_with_metadata: FlowMessageWithMetadata = FlowMessageWithMetadata {
                    flow_message: deserialized_msg,
                    timestamp: Utc::now(),
                    host: "host".into(),
                    malicious: None,
                    id: msg.offset(),
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
}

#[async_trait]
impl Transport for KafkaState {
    async fn consume(&self) {
        info!(
            "Starting to consume messages from kafka [topic: {}]",
            self.topic
        );
        let mut streamer = self.consumer.stream();

        while let Some(event) = streamer.next().await {
            match event {
                Ok(ev) => {
                    let response = self.send_to_actor(ev.detach()).await;
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

#[async_trait]
impl TopicOffsetKeeper for OffsetGuard {
    fn stash_processed_offset(&mut self, offset: i64) {
        self.processed_offests.clone().lock().unwrap().push(offset)
    }

    async fn inc_offset(&mut self) {
        let heap_arc_clone = self.processed_offests.clone();

        loop {
            sleep(Duration::from_secs(OFFSET_COMMIT_INTERVAL)).await;

            let mut inner_heap = heap_arc_clone.lock().unwrap();
            if let Some(smallest_processed_offset) = inner_heap.peek() {
                if smallest_processed_offset * -1 - self.last_stored_offest == 1 {
                    match self.store_offset(smallest_processed_offset * -1) {
                        Ok(()) => self.last_stored_offest = inner_heap.pop().unwrap() * -1,
                        Err(_err) => {
                            warn!("Couldn't store offset for topic: {}", self.state.topic);
                            continue;
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Acknowleger for OffsetGuard {
    async fn end_processing(&self, _id: i64) {
        todo!()
    }
}
