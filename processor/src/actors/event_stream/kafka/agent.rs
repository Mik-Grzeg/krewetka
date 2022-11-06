use super::super::Transport;
use super::context::CustomContext;
use super::get_consumer;
use super::get_producer;

use super::offset_guard::ConsumerOffsetGuard;

use crate::actors::broker::Broker;

use crate::actors::messages::{FlowMessageMetadata, FlowMessageWithMetadata};
use crate::pb::FlowMessage;

use tokio::sync::mpsc;

use async_trait::async_trait;
use log::{debug, error, info};
use prost::Message as PBMessage;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::{Message, OwnedHeaders, OwnedMessage};
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use tokio::sync::Mutex as TokioMtx;

use std::sync::Arc;

use tokio::time::{sleep, Duration};

pub struct KafkaProcessingAgent {
    producer: FutureProducer,
    consumer: Arc<StreamConsumer<CustomContext>>,
    consumer_guard: ConsumerOffsetGuard,
}

impl KafkaProcessingAgent {
    pub fn new(consumer_topic: &str, brokers: &str) -> Self {
        let producer = get_producer(brokers);
        let consumer = get_consumer(brokers);

        consumer
            .subscribe(&[consumer_topic])
            .unwrap_or_else(|_| panic!("Unable to subscribe to topic {}", consumer_topic));

        let consumer_guard = ConsumerOffsetGuard::new(&consumer, consumer_topic);
        info!("created kafka processing agent");

        Self {
            producer,
            consumer: Arc::new(consumer),
            consumer_guard,
        }
    }

    async fn send_to_actor(msg: OwnedMessage, broker: &Arc<TokioMtx<Broker>>) {
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
                broker.lock().await.issue_async(msg_with_metadata);
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
impl Transport for KafkaProcessingAgent {
    async fn guard_acks(&self) {
        self.consumer_guard.inc_offset(&self.consumer).await
    }

    fn ack(&self, offset: i64) {
        self.consumer_guard.stash_processed_offset(offset)
    }

    async fn consume(&self, broker: Arc<TokioMtx<Broker>>, mut notify_rx: mpsc::Receiver<usize>) {
        info!(
            "Starting to consume messages from kafka [topic: {}]",
            self.consumer_guard.topic
        );

        let mut counter: usize = 0;
        while let Some(capacity) = notify_rx.recv().await {
            info!("Received capacity: {}", capacity);
            while counter < capacity {
                let event = match self.consumer.recv().await {
                    Ok(e) => e,
                    Err(e) => {
                        error!("Error: {}", e);
                        sleep(Duration::from_secs(4)).await;
                        continue;
                    }
                };

                Self::send_to_actor(event.detach(), &broker).await;
                counter += 1;
            }
            counter = 0;
        }
    }

    async fn produce(&self, topic: &str, _brokers: &str, msg: &FlowMessageWithMetadata) {
        let mut buffer: Vec<u8> = Vec::with_capacity(4092);

        if let Err(e) = msg.flow_message.encode(&mut buffer) {
            error!("unable to encode message to proto buf format: {}", e);
        }

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
