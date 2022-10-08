use crate::pb::FlowMessage;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait Transport: Send + Sync {
    async fn consume(&self, tx: tokio::sync::broadcast::Sender<FlowMessage>);
    async fn consume_batch(&self, tx: tokio::sync::broadcast::Sender<FlowMessageWithMetadata>);
}

#[derive(Clone, Debug)]
pub struct FlowMessageWithMetadata {
    pub flow_message: FlowMessage,
    pub timestamp: DateTime<Utc>,
    pub host: String,
    pub malicious: Option<bool>,
    // id:             Uuid,
}
