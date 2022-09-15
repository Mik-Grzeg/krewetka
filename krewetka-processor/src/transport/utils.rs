use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn consume(&self, tx: Sender<Vec<u8>>);
}