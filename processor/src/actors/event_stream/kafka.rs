use super::Transport;

use super::consts::OFFSET_COMMIT_INTERVAL;

use crate::actors::messages::{FlowMessageMetadata, FlowMessageWithMetadata};

use actix_broker::{Broker, SystemBroker};
use log::warn;
use rdkafka::error::KafkaError;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use std::str::Utf8Error;
use std::sync::Mutex;
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
    consumer::{Consumer, StreamConsumer},
    ClientConfig, Message, TopicPartitionList,
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
    async fn consume(&self, topic: String, brokers: String) {
        info!("Starting to consume messages from kafka [topic: {}]", topic);
        let consumer = KafkaState::get_consumer(&brokers);
        if let Err(e) = consumer.subscribe(&[&topic]) {
            error!("unable to subscribe to {}\nerror: {}", topic, e)
        };

        let mut streamer = consumer.stream();

        while let Some(event) = streamer.next().await {
            match event {
                Ok(ev) => {
                    let response = Self::send_to_actor(ev.detach()).await;
                    debug!("Actor response: {:?}", response);
                }
                Err(e) => {
                    error!("Unable to receive kafka event: {}", e);
                    continue;
                }
            }
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
            _ => info!("message produced to topic {}", topic),
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
    consumer: Arc<StreamConsumer>,
}

impl ConsumerOffsetGuard {
    pub fn new(
        consumer: &Arc<StreamConsumer>,
        topic: &str,
        processed_offsets: &Arc<Mutex<BinaryHeap<i64>>>,
    ) -> Self {
        consumer
            .subscribe(&[topic])
            .unwrap_or_else(|_| panic!("Unable to subscribe to topic {}", topic));

        let consumer = Arc::clone(consumer);
        let last_stored_offset = ConsumerOffsetGuard::get_offset_for_topic(&consumer, topic);

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

    pub async fn inc_offset(&mut self, topic: String) {
        let heap = self.processed_offsets.clone();

        loop {
            sleep(Duration::from_secs(OFFSET_COMMIT_INTERVAL)).await;

            if let Some(smallest_processed_offset) = heap.lock().unwrap().peek() {
                if smallest_processed_offset * -1 - self.last_stored_offset == 1 {
                    match self.store_offset(smallest_processed_offset * -1) {
                        Ok(()) => self.last_stored_offset = -heap.lock().unwrap().pop().unwrap(),
                        Err(_err) => {
                            warn!("Couldn't store offset for topic: {}", topic);
                            continue;
                        }
                    }
                }
            }
        }
    }

    fn store_offset(&self, offset: i64) -> Result<(), KafkaError> {
        self.consumer.store_offset(&self.topic, 0, offset)
    }

    fn get_offset_for_topic(consumer: &Arc<StreamConsumer>, topic: &str) -> i64 {
        let mut tpl = TopicPartitionList::new();
        let _ = tpl.add_topic_unassigned(topic);

        for _ in 0..3 {
            match consumer.committed_offsets(tpl.clone(), Duration::from_secs(3)) {
                Ok(list) => {
                    let elemts = list.elements_for_topic(topic);
                    if elemts.len() == 1 {
                        return elemts[0].offset().to_raw().unwrap();
                    } else {
                        panic!("Topic is missing: {}", topic);
                    }
                }
                Err(_err) => {
                    std::thread::sleep(Duration::from_secs(3));
                }
            };
        }
        panic!("Could not get offset for topic");
    }
}

pub struct KafkaState {
    brokers: String, // comma separated brokers
}

impl KafkaState {
    pub fn get_consumer(brokers: &str) -> StreamConsumer {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("enable.auto.offset.store", "false")
            .set("group.id", "kafka-krewetka-group")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create()
            .expect("Kafka consumer creation error");
        // consumer.subscribe(&[topic]) // TODO handle properly
        //     .expect(&format!("unable to acquire consumer for topic: {} borkers: {}", topic, brokers));

        consumer
    }

    pub fn get_producer(brokers: &str) -> FutureProducer {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers.clone())
            .set("message.timeout.ms", "5000")
            .set("enable.auto.commit", "true")
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
