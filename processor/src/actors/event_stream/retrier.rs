use super::reader::Transport;
use actix::clock::{sleep_until, Instant};
use rdkafka::{consumer::Consumer, producer::FutureRecord, Message};

use super::consts::BASE_RETRY_INTERVAL_IN_SECS;

use crate::actors::messages::{FlowMessageMetadata, FlowMessageWithMetadata};
use async_trait::async_trait;
use log::error;
use log::info;
use log::warn;

use super::kafka::KafkaState;
use log::debug;
use rdkafka::message::OwnedHeaders;
use std::sync::Arc;
use tokio::time::Duration;
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct Retrier {
    topic_original: String,
    topic_retry: String,
    topic_dlq: String,
    max_retries: usize,
    brokers: String,
}

impl Retrier {
    pub fn new(
        topic_original: String,
        topic_retry: String,
        topic_dlq: String,
        max_retries: usize,
        brokers: String,
    ) -> Self {
        Self {
            topic_original,
            topic_retry,
            topic_dlq,
            max_retries,
            brokers,
        }
    }
}

impl Default for Retrier {
    fn default() -> Self {
        Self {
            topic_original: "flows".to_owned(),
            topic_retry: "flows_retry_".to_owned(),
            topic_dlq: "flows_dead_letter_queue".to_owned(),
            max_retries: 2,
            brokers: "broker:9092".to_owned(),
        }
    }
}

pub struct ArcedRetrier(Arc<Retrier>);

impl ArcedRetrier {
    pub fn new(retrier: Arc<Retrier>) -> Self {
        Self(retrier)
    }

    async fn run_retrier(&self, retry: usize) {
        let consumer = KafkaState::get_consumer(&self.0.brokers);
        let producer = KafkaState::get_producer(&self.0.brokers);
        let destination_topic = match self.0.get_topic_based_on_retry(retry) {
            Some(s) => s,
            None => return,
        };

        consumer.subscribe(&[&destination_topic]);

        let mut stream = consumer.stream();
        let retry_interval = Duration::from_secs(BASE_RETRY_INTERVAL_IN_SECS.pow(retry as u32));
        info!("Starting self.0 {}", destination_topic);

        while let Some(event) = stream.next().await {
            match event {
                Ok(ev) => {
                    let hdrs = ev.headers().unwrap(); // TODO make headers as From<OwnedHeaders> for
                                                      // FlowMessageMetadata
                    let mut metadata = FlowMessageMetadata::try_from(hdrs).unwrap();
                    metadata.offset = Some(ev.offset());

                    if let Some(x) = ev.timestamp().to_millis() {
                        debug!("Sleeping for .. {}", x);
                        let mut deadline = Instant::now() - Duration::from_millis(x as u64);
                        deadline += retry_interval;
                        sleep_until(deadline).await;

                        if let Err((e, _)) = producer
                            .send(
                                FutureRecord::to(&self.0.topic_original)
                                    .payload(ev.payload().unwrap())
                                    .key("KREWETKA")
                                    .headers(
                                        OwnedHeaders::new()
                                            .add("host-identifier-x", &metadata.host)
                                            .add("message-id-x", &metadata.id)
                                            .add("timestamp-x", &metadata.timestamp.to_string())
                                            .add("retry-x", &metadata.retry.to_string()),
                                    ),
                                Duration::from_secs(0),
                            )
                            .await
                        {
                            error!(
                                "Error occured while trying to produce message on retry topic: {}",
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Unable to receive failed message from topic: {}\n{}",
                        destination_topic, e
                    )
                }
            }
        }
    }

    pub async fn init_retry(&self) {
        info!("Spawning retriers");
        tokio::join!(
            self.clone().run_retrier(1),
            self.clone().run_retrier(2),
            self.clone().run_retrier(3),
        );
    }
}

impl Retrier {
    pub fn get_topic_based_on_retry(&self, current_retry: usize) -> Option<String> {
        let max_retries = self.max_retries + 1;
        match (current_retry == max_retries, current_retry > max_retries) {
            (false, false) => Some(format!("{}{}", self.topic_retry, current_retry)),
            (true, false) => Some(self.topic_dlq.to_owned()),
            (_, true) => None,
        }
    }

    pub fn get_brokers_retry(&self) -> &str {
        &self.brokers
    }
}

#[async_trait]
impl Transport for Retrier {
    async fn consume(&self, _topic: String, _brokers: String) {
        todo!()
    }

    async fn produce(&self, _topic: &str, _brokers: &str, _msg: &FlowMessageWithMetadata) {
        todo!()
    }
}
