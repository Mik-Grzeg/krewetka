use super::consts::{STORAGE_BUFFER_FLUSH_INTEVAL_IN_SECS, STORAGE_MAX_BUFFER_SIZE};

use tokio::time::interval;

use crate::actors::broker::Broker;
use crate::actors::event_stream::messages::FlushCollectedEventsToPipeline;
use crate::actors::messages::AckMessage;
use crate::actors::messages::FlowMessageWithMetadata;
use crate::actors::messages::PersistFlowMessageWithMetadata;
use actix::Actor;

use futures::stream::StreamExt;
use log::error;

use actix::Context;
use actix::Handler;
use actix::ResponseFuture;

use actix_broker::BrokerSubscribe;
use async_trait::async_trait;

use log::info;
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMtx;

use super::super::consts::MAILBOX_CAPACITY;
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
    pub broker: Arc<TokioMtx<Broker>>,
}

impl<S> StorageActor<S>
where
    S: AStorage,
{
    pub fn new(storage: Arc<S>, broker: Arc<TokioMtx<Broker>>) -> Self {
        let buffer = Arc::new(Mutex::new(Vec::with_capacity(STORAGE_MAX_BUFFER_SIZE)));

        Self {
            storage,
            buffer,
            broker,
        }
    }
}

impl<S> Actor for StorageActor<S>
where
    S: AStorage + Unpin,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started storage actor!");
        ctx.set_mailbox_capacity(MAILBOX_CAPACITY);

        self.subscribe_async::<BrokerType, PersistFlowMessageWithMetadata>(ctx);
        self.subscribe_async::<BrokerType, InitFlusher>(ctx);

        tokio::spawn({
            let broker = self.broker.clone();
            async move {
                broker.lock().await.issue_async(InitFlusher);
            }
        });
        info!("[storage actor] subscribe to desired kind of messages");
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

async fn after_stash_action(broker: &Arc<TokioMtx<Broker>>, msgs: Vec<AckMessage>) -> usize {
    let msgs_len = msgs.len();
    let mut broker = broker.lock().await;
    for m in msgs.into_iter() {
        broker.issue_async(m);
    }
    msgs_len
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
        let broker = self.broker.clone();

        Box::pin(async move {
            loop {
                interval.tick().await;
                let messages_to_save = buffer
                    .lock()
                    .unwrap()
                    .drain(..)
                    .collect::<Vec<FlowMessageWithMetadata>>();

                info!(
                    "saved batch processing rps: {}",
                    messages_to_save.len() as u64 / STORAGE_BUFFER_FLUSH_INTEVAL_IN_SECS.as_secs()
                );

                if !messages_to_save.is_empty() {
                    let capacity_freed = match storage.stash(messages_to_save).await {
                        Ok(s) => after_stash_action(&broker, s).await,
                        Err(StorageError::DatabaseSave((e, s))) => {
                            error!("failed to save batch: {:?}", e);
                            after_stash_action(&broker, s).await
                        }
                        Err(_) => {
                            panic!("it is imposible to be here")
                        }
                    };
                    info!("storage buffer freed: {capacity_freed:?}");
                    broker
                        .lock()
                        .await
                        .issue_async(FlushCollectedEventsToPipeline(capacity_freed));
                }
            }
        })
    }
}
