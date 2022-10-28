use async_trait::async_trait;

use crate::actors::messages::FlowMessageWithMetadata;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn consume(&self);
    async fn produce(&self, topic: &str, brokers: &str, msg: &FlowMessageWithMetadata);
    async fn guard_acks(&self);
    fn ack(&self, id: i64);
}

#[async_trait]
pub trait RetrierExt: Send + Sync {
    async fn spawn_retriers(&self);
    fn get_topic_based_on_retry(&self, current_retry: usize) -> Option<String>;
    fn get_brokers_retry(&self) -> &str;
}
