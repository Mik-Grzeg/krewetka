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

pub fn get_current_offset(consumer: &Arc<StreamConsumer<CustomContext>>, topic: &str) -> Offset {
    // match consumer.assignment() {
    //     Ok(a) => {
    //         info!("assigments for topic {topic}: {a:?}");
    //         match a.elements_for_topic(topic).first() {
    //             Some(e) =>  e.offset(),
    //             _ => continue
    //         }
    //     }
    //     _ => continue
    // }

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
    pub fn new(_consumer: Arc<StreamConsumer<CustomContext>>, topic: &str) -> Self {
        let heap: Arc<Mutex<BinaryHeap<i64>>> = Arc::new(Mutex::new(BinaryHeap::new()));

        // let last_stored_offset = match get_current_offset(&consumer, topic) {
        //     Offset::Invalid => 0,
        //     Offset::Offset(x) => x,
        //     c => panic!("unknown offset: {:?}", c),
        // };

        Self {
            processed_offsets: heap,
            last_stored_offset: Mutex::new(0),
            topic: topic.to_owned(),
        }
    }

    pub fn stash_processed_offset(
        &self,
        consumer: &Arc<StreamConsumer<CustomContext>>,
        offset: i64,
        partition: i32,
    ) {
        // self.processed_offsets.clone().lock().unwrap().push(-offset)
        if let Err(_e) = consumer.store_offset(&self.topic, partition, offset) {
            error!("error occured while trying to store Offset({offset}) to [{topic}] at partition {partition}", topic=self.topic);
        };
    }

    fn peek_heap(&self, peek: &Option<&i64>, last_stored_offset: &i64) -> bool {
        match *peek {
            Some(x) => 1 >= (x * -1 - last_stored_offset),
            None => false,
        }
    }

    async fn tt<'a>(&self, consumer: &Arc<StreamConsumer<CustomContext>>) {
        let mut ready = false;

        while !ready {
            sleep(Duration::from_secs(2)).await;
            // let mut c: TopicPartitionList;
            // let mut partition: Vec<TopicPartitionListElem>;

            let c = match consumer.committed(rdkafka::util::Timeout::Never) {
                Ok(l) => l,
                Err(_) => {
                    warn!("continued at consumer.committed()");
                    continue;
                }
            };
            let partition = c.elements_for_topic(&self.topic);

            // partition = c.elements_for_topic(&self.topic);

            if partition.first().is_none() {
                warn!("continued at partition.first() for topic {}", self.topic);
                continue;
            }

            for elem in partition.iter() {
                info!(
                    "partition for [{}]: {:?} with offset {:?}",
                    self.topic,
                    elem.partition(),
                    elem.offset()
                );
            }
            let partition1 = partition.into_iter().next().unwrap();

            {
                let mut last_stored_offset = self.last_stored_offset.lock().unwrap();
                let fetched_offset = partition1.offset();

                info!("Fetched offset: {fetched_offset:?}");
                *last_stored_offset = match fetched_offset {
                    Offset::Offset(o) => o,
                    Offset::Invalid => 0,
                    o => panic!(
                        "Unexpected offset {:?} for {}/{}",
                        o,
                        self.topic,
                        partition1.partition()
                    ),
                }
            }
            ready = true;
        }
    }

    pub async fn inc_offset(&self, _consumer: Arc<StreamConsumer<CustomContext>>) {
        // let heap = self.processed_offsets.clone();
        // info!(
        //     "Spawned offset incrementer for {} with current offset {:?}",
        //     self.topic,
        //     self.last_stored_offset.lock().unwrap()
        // );

        // self.tt(&consumer).await;

        // loop {

        //     sleep(Duration::from_secs(2)).await;

        //     info!("{:?}", heap.lock().unwrap());
        //     let mut heap_peek = heap.lock().unwrap();

        //     let mut last_stored_offset = self.last_stored_offset.lock().unwrap();
        //     while self.peek_heap(&heap_peek.peek(), &last_stored_offset) {
        //         *last_stored_offset = -heap_peek.pop().unwrap();
        //     }

        //     // make it into a stream with a timeout to commit
        //     self.store_offset(*last_stored_offset, &consumer)
        // }
    }

    fn store_offset(&self, offset: i64, consumer: &Arc<StreamConsumer<CustomContext>>) {
        let mut tpl = TopicPartitionList::new();

        let assignment = consumer.assignment().unwrap();
        info!("assignment: {:?}", assignment);
        info!("Offset to store: {} for topic {}", offset, self.topic);
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
