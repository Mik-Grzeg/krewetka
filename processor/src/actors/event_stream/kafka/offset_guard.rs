use rdkafka::consumer::CommitMode;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::StreamConsumer;
use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};

use rdkafka::Offset;
use rdkafka::TopicPartitionList;

use super::context::CustomContext;
use log::*;
use tokio::time::{sleep, Duration};

pub fn get_current_offset(consumer: &StreamConsumer<CustomContext>, topic: &str) -> Offset {
    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition(topic, 0);

    match consumer.committed_offsets(tpl.clone(), Duration::from_secs(10)) {
        Ok(list) => {
            let topic_map = list.to_topic_map();
            topic_map.get(&(topic.to_owned(), 0)).unwrap().to_owned()
        }
        Err(err) => {
            error!("mega error while checking offset: {}", err);
            match err {
                KafkaError::OffsetFetch(RDKafkaErrorCode::UnknownTopicOrPartition) => {
                    Offset::Beginning
                }
                _ => Offset::Invalid,
            }
        }
    }
}

pub struct ConsumerOffsetGuard {
    pub processed_offsets: Arc<Mutex<BinaryHeap<i64>>>,
    pub last_stored_offset: Mutex<i64>,
    pub topic: String,
}

impl ConsumerOffsetGuard {
    pub fn new(consumer: &StreamConsumer<CustomContext>, topic: &str) -> Self {
        let heap: Arc<Mutex<BinaryHeap<i64>>> = Arc::new(Mutex::new(BinaryHeap::new()));

        let last_stored_offset = match get_current_offset(consumer, topic) {
            Offset::Invalid => 0,
            Offset::Offset(x) => x,
            c => panic!("unknown offset: {:?}", c),
        };

        Self {
            processed_offsets: heap,
            last_stored_offset: Mutex::new(last_stored_offset),
            topic: topic.to_owned(),
        }
    }

    pub fn stash_processed_offset(&self, offset: i64) {
        self.processed_offsets.clone().lock().unwrap().push(-offset)
    }

    fn peek_heap(&self, peek: &Option<&i64>, last_stored_offset: &i64) -> bool {
        match *peek {
            Some(x) => 1 >= (x * -1 - last_stored_offset),
            None => false,
        }
    }

    pub async fn inc_offset(&self, consumer: &StreamConsumer<CustomContext>) {
        let heap = self.processed_offsets.clone();
        info!(
            "Spawned offset incrementer for {} with current offset {:?}",
            self.topic,
            self.last_stored_offset.lock().unwrap()
        );

        loop {
            sleep(Duration::from_secs(2)).await;

            let mut heap_peek = heap.lock().unwrap();

            let mut last_stored_offset = self.last_stored_offset.lock().unwrap();
            while self.peek_heap(&heap_peek.peek(), &last_stored_offset) {
                *last_stored_offset = -heap_peek.pop().unwrap();
            }

            // make it into a stream with a timeout to commit
            self.store_offset(*last_stored_offset, consumer)
        }
    }

    fn store_offset(&self, offset: i64, consumer: &StreamConsumer<CustomContext>) {
        let mut tpl = TopicPartitionList::new();

        debug!("Offset to store: {}", offset);
        if let Err(e) = tpl.add_partition_offset(&self.topic, 0, Offset::from_raw(offset)) {
            error!("Something odd for the topic partition offset: {}", e);
        };

        if let Err(KafkaError::OffsetFetch(RDKafkaErrorCode::UnknownTopicOrPartition)) =
            consumer.commit(&tpl, CommitMode::Sync)
        {
            debug!(" unable to commit because topic or partition does not exists");
        }
    }
}
