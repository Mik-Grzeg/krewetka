use super::consts::{STORAGE_BUFFER_FLUSH_INTEVAL_IN_SECS, STORAGE_MAX_BUFFER_SIZE};
use futures::task::noop_waker;
use tokio::time::interval;

use crate::actors::messages::FlowMessageWithMetadata;
use crate::actors::messages::PersistFlowMessageWithMetadata;
use actix::Actor;

use std::task::Context as TaskCtx;

use actix::Context;
use actix::Handler;
use actix::ResponseFuture;
use actix_broker::BrokerSubscribe;
use async_trait::async_trait;
use clickhouse_rs::errors::Error as ChError;
use log::debug;
use log::{info, warn};
use std::error::Error;
use std::sync::Arc;

use tokio::sync::mpsc;

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
pub trait AStorage: 'static {
    async fn stash(&self, msgs: Vec<FlowMessageWithMetadata>) -> Result<(), StorageError>;
}

pub struct StorageActor<S>
where
    S: AStorage,
{
    storage: Arc<S>,
    buffer_channel_sender: mpsc::Sender<FlowMessageWithMetadata>,
    // pub broker: Arc<Mutex<Broker>>
}

impl<S> StorageActor<S>
where
    S: AStorage,
{
    pub fn new(storage: Arc<S>) -> (Self, mpsc::Receiver<FlowMessageWithMetadata>) {
        let (tx, rx) = mpsc::channel::<FlowMessageWithMetadata>(STORAGE_MAX_BUFFER_SIZE);

        let slf = Self {
            storage,
            buffer_channel_sender: tx,
        };

        (slf, rx)
    }
}

impl<S> Actor for StorageActor<S>
where
    S: AStorage,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started storage actor!");

        self.subscribe_async::<BrokerType, PersistFlowMessageWithMetadata>(ctx)
    }
}

impl<S> Handler<PersistFlowMessageWithMetadata> for StorageActor<S>
where
    S: AStorage,
{
    type Result = ResponseFuture<()>;

    fn handle(
        &mut self,
        msg: PersistFlowMessageWithMetadata,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let tx = self.buffer_channel_sender.clone();

        Box::pin(async move {
            if let Err(e) = tx.send(msg.0).await {
                // TODO create retry event
                warn!("unable to send message to storage buffer: {}", e)
            }
        })
    }
}

pub async fn flush_buffer<S: AStorage>(
    storage: Arc<S>,
    mut buffer_recv: mpsc::Receiver<FlowMessageWithMetadata>,
) {
    let mut buffer: Vec<FlowMessageWithMetadata> = Vec::with_capacity(STORAGE_MAX_BUFFER_SIZE);
    let mut interval = interval(STORAGE_BUFFER_FLUSH_INTEVAL_IN_SECS);
    let waker = noop_waker();
    let mut ctx = TaskCtx::from_waker(&waker);

    while let Some(msg) = buffer_recv.recv().await {
        buffer.push(msg);

        if interval.poll_tick(&mut ctx).is_ready() {
            let messages_to_save = buffer.drain(..).collect::<Vec<FlowMessageWithMetadata>>();
            info!("messages_to_save len {}, last batch speed {} rps", messages_to_save.len(), messages_to_save.len() as u64/STORAGE_BUFFER_FLUSH_INTEVAL_IN_SECS.as_secs());

            if let Err(e) = storage.stash(messages_to_save).await {
                debug!("Failed to save messages: {:?}", e);
            }
        }
    }
}
