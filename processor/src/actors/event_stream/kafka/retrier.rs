use super::super::transport::RetrierExt;
use super::consts::*;
use super::get_consumer;
use super::get_producer;
use crate::actors::messages::FlowMessageMetadata;
use async_trait::async_trait;
use futures::StreamExt;
use log::*;
use rdkafka::consumer::Consumer;
use rdkafka::message::Message;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::FutureRecord;
use tokio::time::{sleep_until, Duration, Instant};

#[async_trait]
impl RetrierExt for Retrier {
    async fn spawn_retriers(&self) {
        info!("Spawning retriers");
        tokio::join!(
            self.clone().run_retrier(1),
            self.clone().run_retrier(2),
            self.clone().run_retrier(3),
        );
    }

    fn get_topic_based_on_retry(&self, current_retry: usize) -> Option<String> {
        let max_retries = self.max_retries + 1;
        match (current_retry == max_retries, current_retry > max_retries) {
            (false, false) => Some(format!("{}{}", self.topic_retry, current_retry)),
            (true, false) => Some(self.topic_dlq.to_owned()),
            (_, true) => None,
        }
    }

    fn get_brokers_retry(&self) -> &str {
        &self.brokers
    }
}

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

impl Retrier {
    async fn run_retrier(&self, retry: usize) {
        info!("Starting mega retrier_{}", retry);
        let consumer = get_consumer(&self.brokers);
        let producer = get_producer(&self.brokers);

        let destination_topic = match self.get_topic_based_on_retry(retry) {
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
                                FutureRecord::to(&self.topic_original)
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
}
