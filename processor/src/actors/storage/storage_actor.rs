use super::consts::RETRY_STORAGE_INSERT_INTERVAL_IN_SECS;
use super::consts::STORAGE_BUFFER_SIZE;
use crate::actors::acker::messages::AckMessage;
use crate::actors::messages::FlowMessageWithMetadata;
use crate::actors::messages::PersistFlowMessageWithMetadata;
use actix::Actor;
use actix_broker::Broker;

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
use tokio::time::sleep;

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
    async fn stash(&self, msgs: Vec<PersistFlowMessageWithMetadata>) -> Result<(), StorageError>;
}

pub struct StorageActor<S>
where
    S: AStorage,
{
    storage: Arc<S>,
    buffer_channel_sender: mpsc::Sender<PersistFlowMessageWithMetadata>,
    // pub broker: Arc<Mutex<Broker>>
}

impl<S> StorageActor<S>
where
    S: AStorage,
{
    pub fn new(storage: Arc<S>) -> (Self, mpsc::Receiver<PersistFlowMessageWithMetadata>) {
        let (tx, rx) = mpsc::channel::<PersistFlowMessageWithMetadata>(STORAGE_BUFFER_SIZE);

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
            if let Err(e) = tx.send(msg).await {
                // TODO create retry event
                warn!("unable to send message to storage buffer: {}", e)
            }
        })
    }
}

pub async fn flush_buffer<S: AStorage>(
    storage: Arc<S>,
    mut buffer_recv: mpsc::Receiver<PersistFlowMessageWithMetadata>,
) {
    let mut buffer: Vec<PersistFlowMessageWithMetadata> = Vec::with_capacity(STORAGE_BUFFER_SIZE);

    let mut last_save_failed = true;
    while let Some(msg) = buffer_recv.recv().await {
        debug!("Got msg: {:?}", msg);
        buffer.push(msg);

        if buffer.len() == STORAGE_BUFFER_SIZE {
            last_save_failed = true;
            for try_save_interval in RETRY_STORAGE_INSERT_INTERVAL_IN_SECS {
                if let Err(e) = storage
                    .stash(
                        buffer
                            .drain(..)
                            .collect::<Vec<PersistFlowMessageWithMetadata>>(),
                    )
                    .await
                {
                    debug!("Failed to save messages: {:?}", e);
                    // TODO should pass those messages to nacking actor
                } else {
                    last_save_failed = false;
                    break;
                }
                sleep(try_save_interval).await;
            }

            if last_save_failed {
                buffer.drain(..).into_iter().for_each(|f| {
                    Broker::<BrokerType>::issue_async(AckMessage::NackRetry(
                        FlowMessageWithMetadata::from(f),
                    ))
                });
            }
        }
    }
}
