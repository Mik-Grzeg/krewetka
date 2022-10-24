use super::Transport;

use super::consts::OFFSET_COMMIT_INTERVAL;

use crate::actors::messages::{FlowMessageMetadata, FlowMessageWithMetadata};


use actix_broker::{Broker, SystemBroker};
use log::warn;
use rdkafka::Offset;
use rdkafka::consumer::CommitMode;
use rdkafka::error::KafkaError;
use rdkafka::error::KafkaResult;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use core::panic;
use std::str::Utf8Error;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::time::Duration;
use tokio::time::sleep;

use crate::pb::FlowMessage;
use std::collections::BinaryHeap;
use std::sync::Arc;

use async_trait::async_trait;
use std::convert::TryFrom;

use log::{debug, error, info};
use prost::Message as PBMessage;
use rdkafka::message::{BorrowedHeaders, FromBytes, Headers, OwnedHeaders, OwnedMessage};
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{Consumer, StreamConsumer, ConsumerContext, Rebalance},
    ClientConfig, Message, TopicPartitionList,
    ClientContext
};

use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct KafkaSettings {
    pub brokers: Vec<String>,
    pub topic: String,
}

pub struct ProcessingAgent {
    producer: FutureProducer,
}

impl ProcessingAgent {
    pub fn new(producer: FutureProducer) -> Self {
        Self { producer }
    }
}

#[async_trait]
impl Transport for ProcessingAgent {
    async fn consume(&self, consumer: Arc<StreamConsumer<CustomContext>>, topic: String, brokers: String) {
        info!("Starting to consume messages from kafka [topic: {}]", topic);

        let partition = 0;
        let starting_offset = match get_offset_for_topic(&consumer, &topic) {
            Offset::Invalid => Offset::from_raw(0),
            offset => offset,
        };

        if let Err(e) = consumer.seek(&topic, partition, starting_offset, Duration::from_secs(3)) {
            error!("unable to seek offset {:?} in topic {} on partition {} with error: {}", starting_offset, topic, partition, e);
        }

        loop {
            let event = match consumer.recv().await {
                Ok(e) => e,
                Err(e) => panic!("Jakis error lol: {}", e)
            };
            debug!("Fetched offset: {}", event.offset());
            let response = Self::send_to_actor(event.detach()).await;
            debug!("Actor response: {:?}", response);
        }
    }

    async fn produce(&self, topic: &str, _brokers: &str, msg: &FlowMessageWithMetadata) {
        let mut buffer: Vec<u8> = Vec::with_capacity(4092);

        if let Err(e) = msg.flow_message.encode(&mut buffer) {
            error!("unable to encode message to proto buf format: {}", e);
        }

        info!("Message sent to retry topic: {}", topic);
        match self
            .producer
            .clone()
            .send(
                FutureRecord::to(topic)
                    .payload(&buffer)
                    .key("KREWETKA")
                    .headers(
                        OwnedHeaders::new()
                            .add("host-identifier-x", &msg.metadata.host)
                            .add("message-id-x", &msg.metadata.id)
                            .add("timestamp-x", &msg.metadata.timestamp.to_string())
                            .add("retry-x", &msg.metadata.retry.to_string()),
                    ),
                Duration::from_secs(0),
            )
            .await
        {
            Err((err, msg)) => {
                error!("unable to produce message: {:?}\nerror: {}", msg, err)
            }
            _ => debug!("message produced to topic {}", topic),
        }
    }
}

#[derive(Debug)]
pub enum EventStreamError {
    UnableToParseKafkaHeader(String),
}

impl From<Utf8Error> for EventStreamError {
    fn from(e: Utf8Error) -> Self {
        EventStreamError::UnableToParseKafkaHeader(e.to_string())
    }
}

impl FlowMessageMetadata {
    pub fn map_hdr<T>(r: Option<(&str, T)>, hdr: &str) -> Result<T, EventStreamError> {
        match r {
            Some((_h, v)) => Ok(v),
            None => Err(EventStreamError::UnableToParseKafkaHeader(hdr.to_owned())),
        }
    }
}

impl TryFrom<&OwnedHeaders> for FlowMessageMetadata {
    type Error = EventStreamError;

    fn try_from(headers: &OwnedHeaders) -> Result<Self, Self::Error> {
        let host = str::from_bytes(FlowMessageMetadata::map_hdr(
            headers.get(0),
            "host-identifier-x",
        )?)?
        .to_owned();
        let id = str::from_bytes(FlowMessageMetadata::map_hdr(
            headers.get(1),
            "message-id-x",
        )?)?
        .to_owned();
        let retry = str::from_bytes(FlowMessageMetadata::map_hdr(headers.get(3), "retry-x")?)?
            .to_owned()
            .parse::<usize>()
            .unwrap();
        let timestamp =
            str::from_bytes(FlowMessageMetadata::map_hdr(headers.get(2), "timestamp-x")?)?
                .to_owned()
                .parse::<u64>()
                .unwrap();

        Ok(FlowMessageMetadata {
            host,
            id,
            timestamp,
            retry,
            offset: None,
        })
    }
}

impl TryFrom<&BorrowedHeaders> for FlowMessageMetadata {
    type Error = EventStreamError;

    fn try_from(headers: &BorrowedHeaders) -> Result<Self, Self::Error> {
        let host = str::from_bytes(FlowMessageMetadata::map_hdr(
            headers.get(0),
            "host-identifier-x",
        )?)?
        .to_owned();
        let id = str::from_bytes(FlowMessageMetadata::map_hdr(
            headers.get(1),
            "message-id-x",
        )?)?
        .to_owned();
        let retry = str::from_bytes(FlowMessageMetadata::map_hdr(headers.get(3), "retry-x")?)?
            .to_owned()
            .parse::<usize>()
            .unwrap();
        let timestamp =
            str::from_bytes(FlowMessageMetadata::map_hdr(headers.get(2), "timestamp-x")?)?
                .to_owned()
                .parse::<u64>()
                .unwrap();

        Ok(FlowMessageMetadata {
            host,
            id,
            timestamp,
            retry,
            offset: None,
        })
    }
}

impl ProcessingAgent {
    async fn send_to_actor(msg: OwnedMessage) {
        let hdrs = msg.headers().unwrap(); // TODO make headers as From<OwnedHeaders> for
                                           // FlowMessageMetadata
        let mut metadata: FlowMessageMetadata = hdrs.try_into().unwrap();
        metadata.offset = Some(msg.offset());

        match msg.payload_view::<[u8]>() {
            Some(Ok(f)) => {
                let deserialized_msg: FlowMessage = PBMessage::decode::<&[u8]>(f).unwrap(); // TODO properly handle error

                let msg_with_metadata: FlowMessageWithMetadata = FlowMessageWithMetadata {
                    flow_message: deserialized_msg,
                    malicious: None,
                    metadata,
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

// pub struct CustomContext;

// impl ClientContext for CustomContext {}

// impl ConsumerContext for CustomContext {
//     fn pre_rebalance(&self, rebalance: &Rebalance) {
//         info!("Pre rebalance {:?}", rebalance);
//     }

//     fn post_rebalance(&self, rebalance: &Rebalance) {
//         info!("Post rebalance {:?}", rebalance);
//     }

//     fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
//         info!("Committing offsets: {:?}", result);
//     }
// }

pub struct ConsumerOffsetGuard {
    processed_offsets: Arc<Mutex<BinaryHeap<i64>>>,
    last_stored_offset: i64,
    topic: String,
    consumer: Arc<StreamConsumer<CustomContext>>,
}

impl ConsumerOffsetGuard {
    pub fn new(
        consumer: Arc<StreamConsumer<CustomContext>>,
        topic: &str,
        processed_offsets: Arc<Mutex<BinaryHeap<i64>>>,
    ) -> Self {

        let consumer = Arc::clone(&consumer);
        let last_stored_offset = match get_offset_for_topic(&consumer, topic) {
            Offset::Invalid => -1,
            Offset::Offset(x) => x,
            c => panic!("weird offset: {:?}", c)
        };

        Self {
            processed_offsets: processed_offsets.clone(),
            last_stored_offset,
            consumer,
            topic: topic.to_owned(),
        }
    }

    pub fn stash_processed_offset(&mut self, offset: i64) {
        self.processed_offsets.clone().lock().unwrap().push(offset)
    }

    fn peek_heap(&self, peek: &Option<&i64>) -> bool {
        match *peek {
            Some(x) => {
                match x * -1 - self.last_stored_offset {
                    1 | 0 => true,
                    _ => false
                }
            },
            None => false
        } 
    }

    pub async fn inc_offset(&mut self, topic: String) {
        let heap = self.processed_offsets.clone();

        loop {
            sleep(Duration::from_secs(2)).await;

            let mut heap_peek = heap.lock().unwrap();

            while self.peek_heap(&heap_peek.peek()) {
                self.last_stored_offset = -1 * heap_peek.pop().unwrap();  
            } 

            // make it into a stream with a timeout to commit
            match self.store_offset(self.last_stored_offset) {
                Ok(()) => {
                    info!("Updated last_stored_offset to {}", self.last_stored_offset);
                }
                Err(e) => {
                    warn!("Couldn't store offset for topic: {}", topic);
                    // panic!("Something fucked up: {}", e);
                }
            }

        }
    }

    fn store_offset(&self, offset: i64) -> Result<(), KafkaError> {
        let mut tpl = TopicPartitionList::new();
        if let Err(e) = tpl.add_partition_offset(&self.topic, 0, Offset::from_raw(offset)) {
            error!("STORAGE OFFSET SMTH WRONG: {}", e);
        };

        match self.consumer.commit(&tpl, CommitMode::Sync) {
            Err(e) => {
                error!("unable to store offset: {}", e);
                Err(e)
            }
            Ok(ok) => {
                info!("stored offset: {} for topic {}", offset, &self.topic);
                Ok(ok)
            }
        }
    }
}

fn get_offset_for_topic(consumer: &StreamConsumer<CustomContext>, topic: &str) -> Offset {
    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition(topic, 0);

    for i in 0..3 {
        info!("Offset chjecking iteration {}", i);
        match consumer.committed_offsets(tpl.clone(), Duration::from_secs(3)) {
            Ok(list) => {
                let topic_map = list.to_topic_map();
                debug!("topic map: {:?}", topic_map);
                return topic_map.get(&(topic.to_owned(), 0)).unwrap().to_owned()
            }
            Err(_err) => {
                std::thread::sleep(Duration::from_secs(3));
            }
        };
    }
    panic!("Could not get offset for topic");
}

pub struct KafkaState {
    brokers: String, // comma separated brokers
}

impl KafkaState {
    pub fn get_consumer(brokers: &str) -> StreamConsumer<CustomContext> {
        let ctx = CustomContext;
        let consumer: StreamConsumer<CustomContext> = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            // .set("enable.auto.offset.store", "false")
            .set("auto.offset.reset", "earliest")
            .set("group.id", "krewetka-group")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(ctx)
            .expect("Kafka consumer creation error");
        // consumer.subscribe(&[topic]) // TODO handle properly
        //     .expect(&format!("unable to acquire consumer for topic: {} borkers: {}", topic, brokers));

        consumer
    }

    pub fn get_producer(brokers: &str) -> FutureProducer {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers.clone())
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        producer
    }

    pub fn new(&self, brokers: String) -> Self {
        let _consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers.clone())
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("enable.auto.offset.store", "false")
            .set("enable.offset.reset", "earliest")
            .set("group.id", "kafka-krewetka-group")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create()
            .expect("Kafka consumer creation error");

        let _producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers.clone())
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        KafkaState { brokers }
    }

    pub fn extract_retry_from_headers<T: Headers>(hdrs: &Option<&T>) -> i32 {
        match hdrs {
            Some(hdrs) => {
                if let Some((_, val)) = hdrs.get(0) {
                    i32::from_be_bytes(<[u8; 4]>::try_from(val).unwrap())
                } else {
                    0
                }
            }
            None => 0,
        }
    }
}

// #[async_trait]
// impl Transport for KafkaState {
//     async fn consume(&self, topic: String, brokers: String) {
//         info!(
//             "Starting to consume messages from kafka [topic: {}]",
//             topic
//         );
//         let consumer = Self::get_consumer(&brokers, &topic);

//         let mut streamer = consumer.stream();

//         while let Some(event) = streamer.next().await {
//             match event {
//                 Ok(ev) => {
//                     let response = self.send_to_actor(ev.detach()).await;
//                     debug!("Actor response: {:?}", response);
//                 }
//                 Err(e) => {
//                     error!("Unable to receive kafka event: {}", e);
//                     continue;
//                 }
//             }
//         }
//     }

//     async fn produce(&self, topic: String) {
//         todo!()
//     }
// }

pub struct CustomContext;

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
