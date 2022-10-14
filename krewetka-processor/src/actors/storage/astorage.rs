use crate::actors::messages::PersistFlowMessageWithMetadata;
use actix::Actor;

use actix::Context;
use actix::Handler;
use actix::ResponseFuture;
use actix_broker::BrokerSubscribe;
use async_trait::async_trait;
use clickhouse_rs::errors::Error as ChError;
use log::info;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::clickhouse::ClickhouseState;
use crate::actors::BrokerType;

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
    async fn stash_on_buffer(&self, msg: PersistFlowMessageWithMetadata);
    async fn flush_buffer(&self, mut buffer_recv: mpsc::Receiver<PersistFlowMessageWithMetadata>);
}

pub struct StorageActor {
    pub storage: Arc<ClickhouseState>,
    // pub broker: Arc<Mutex<Broker>>
}

impl Actor for StorageActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started storage actor!");
        self.subscribe_async::<BrokerType, PersistFlowMessageWithMetadata>(ctx)
    }
}

impl Handler<PersistFlowMessageWithMetadata> for StorageActor {
    type Result = ResponseFuture<()>;

    fn handle(
        &mut self,
        msg: PersistFlowMessageWithMetadata,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let state = self.storage.clone();
        Box::pin(async move {
            state.stash_on_buffer(msg).await;
        })
    }
}
