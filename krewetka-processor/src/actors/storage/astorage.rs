use crate::actors::event_reader::FlowMessageWithMetadata;
use crate::actors::messages::MessageInPipeline;
use actix::Actor;
use actix::Context;
use actix::Handler;
use actix::ResponseFuture;
use async_trait::async_trait;
use clickhouse_rs::errors::Error as ChError;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::clickhouse::ClickhouseState;

#[derive(Debug)]
pub enum StorageError {
    Database(Box<dyn Error>),
}

impl From<ChError> for StorageError {
    fn from(e: ChError) -> StorageError {
        StorageError::Database(Box::new(e))
    }
}

#[async_trait]
pub trait AStorage {
    async fn stash_on_buffer(&self, msg: FlowMessageWithMetadata);
    async fn flush_buffer(&self, mut buffer_recv: mpsc::Receiver<FlowMessageWithMetadata>);
}

pub struct StorageActor {
    pub storage: Arc<ClickhouseState>,
}

impl Actor for StorageActor {
    type Context = Context<Self>;
}

impl Handler<MessageInPipeline> for StorageActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: MessageInPipeline, _ctx: &mut Self::Context) -> Self::Result {
        let state = self.storage.clone();
        Box::pin(async move {
            state.stash_on_buffer(msg.0).await;
        })
    }
}
