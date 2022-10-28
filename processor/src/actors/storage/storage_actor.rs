use super::consts::{STORAGE_BUFFER_FLUSH_INTEVAL_IN_SECS, STORAGE_MAX_BUFFER_SIZE};

use tokio::time::interval;

use crate::actors::messages::AckMessage;
use crate::actors::messages::FlowMessageWithMetadata;
use crate::actors::messages::PersistFlowMessageWithMetadata;
use actix::Actor;

use futures::stream::StreamExt;
use log::error;

use actix::Context;
use actix::Handler;
use actix::ResponseFuture;
use actix_broker::Broker;
use actix_broker::BrokerSubscribe;
use async_trait::async_trait;

use log::info;
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;

use super::messages::InitFlusher;

use crate::actors::BrokerType;

#[derive(Debug)]
pub enum StorageError {
    Database(Box<dyn Error>),
    DatabaseSave((Box<dyn Error>, Vec<AckMessage>)),
}

pub type FlowMessageStream = futures::stream::Iter<std::vec::IntoIter<FlowMessageWithMetadata>>;
pub type OffsetStream = futures::stream::Iter<std::vec::IntoIter<i64>>;
#[async_trait]
pub trait AStorage: 'static {
    async fn stash(
        &self,
        msgs: Vec<FlowMessageWithMetadata>,
    ) -> Result<Vec<AckMessage>, StorageError>;
}

pub struct StorageActor<S>
where
    S: AStorage,
{
    storage: Arc<S>,
    buffer: Arc<Mutex<Vec<FlowMessageWithMetadata>>>,
}

impl<S> StorageActor<S>
where
    S: AStorage,
{
    pub fn new(storage: Arc<S>) -> Self {
        let buffer = Arc::new(Mutex::new(Vec::with_capacity(STORAGE_MAX_BUFFER_SIZE)));

        Self { storage, buffer }
    }
}

impl<S> Actor for StorageActor<S>
where
    S: AStorage + Unpin,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started storage actor!");

        self.subscribe_async::<BrokerType, PersistFlowMessageWithMetadata>(ctx);
        self.subscribe_async::<BrokerType, InitFlusher>(ctx);
    }
}

impl<S> Handler<PersistFlowMessageWithMetadata> for StorageActor<S>
where
    S: AStorage + Unpin,
{
    type Result = ();

    fn handle(
        &mut self,
        msg: PersistFlowMessageWithMetadata,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let buffer = self.buffer.clone();

        buffer.lock().unwrap().push(msg.0);
    }
}

impl<S> Handler<InitFlusher> for StorageActor<S>
where
    S: AStorage + Unpin,
{
    type Result = ResponseFuture<()>;

    fn handle(&mut self, _msg: InitFlusher, _ctx: &mut Self::Context) -> Self::Result {
        let mut interval = interval(STORAGE_BUFFER_FLUSH_INTEVAL_IN_SECS);
        let storage = self.storage.clone();
        let buffer = self.buffer.clone();

        Box::pin(async move {
            loop {
                interval.tick().await;
                let messages_to_save = buffer
                    .lock()
                    .unwrap()
                    .drain(..)
                    .collect::<Vec<FlowMessageWithMetadata>>();
                info!(
                    "messages_to_save len {}\nprevious batch processing rps: {}",
                    messages_to_save.len(),
                    messages_to_save.len() as u64 / STORAGE_BUFFER_FLUSH_INTEVAL_IN_SECS.as_secs()
                );

                if !messages_to_save.is_empty() {
                    match storage.stash(messages_to_save).await {
                        Ok(s) => s.into_iter().for_each(Broker::<BrokerType>::issue_async),
                        Err(StorageError::DatabaseSave((e, s))) => {
                            error!("failed to save batch: {:?}", e);
                            s.into_iter().for_each(Broker::<BrokerType>::issue_async)
                        }
                        Err(_) => {
                            panic!("it is imposible to be here")
                        }
                    }
                }
            }
        })
    }
}
