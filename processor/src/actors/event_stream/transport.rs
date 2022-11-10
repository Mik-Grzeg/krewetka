use async_trait::async_trait;

use crate::actors::broker::Broker;
use crate::actors::messages::FlowMessageWithMetadata;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMtx;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn consume(&self, broker: Arc<TokioMtx<Broker>>, notify_rx: mpsc::Receiver<usize>);
    async fn produce(&self, topic: &str, brokers: &str, msg: &FlowMessageWithMetadata);
    async fn guard_acks(&self);
    fn ack(&self, id: i64, partition: i32);
}

#[async_trait]
pub trait RetrierExt: Send + Sync {
    async fn spawn_retriers(&self);
    fn get_topic_based_on_retry(&self, current_retry: usize) -> Option<String>;
    fn get_brokers_retry(&self) -> &str;
}
