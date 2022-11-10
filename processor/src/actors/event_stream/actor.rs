use super::super::BrokerType;
use super::RetrierExt;
use super::Transport;

use actix::AsyncContext;
use actix::WrapFuture;
use actix::{Actor, Context, Handler, ResponseFuture};
use actix_broker::{BrokerIssue, BrokerSubscribe};
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMtx;

use super::super::consts::MAILBOX_CAPACITY;
use super::messages::FlushCollectedEventsToPipeline;
use super::messages::InitConsumer;
use super::messages::InitRetrier;
use crate::actors::broker::Broker;
use crate::actors::messages::AckMessage;
use crate::actors::messages::ClassifyFlowMessageWithMetadata;
use crate::actors::messages::FlowMessageWithMetadata;
use log::*;

pub struct EventStreamActor<T, R> {
    pub processor: Arc<T>,
    pub retrier: Arc<R>,
    pub broker: Arc<TokioMtx<Broker>>,
    pub notify_channel: Option<mpsc::Sender<usize>>,
}

impl<T, R> EventStreamActor<T, R>
where
    T: Transport + 'static,
    R: RetrierExt + 'static,
{
    pub fn new(processor: Arc<T>, retrier: Arc<R>, broker: Arc<TokioMtx<Broker>>) -> Self {
        Self {
            processor,
            retrier,
            broker,
            notify_channel: None,
        }
    }
}

impl<T, R> Actor for EventStreamActor<T, R>
where
    T: Transport + 'static,
    R: RetrierExt + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started transport actor");
        ctx.set_mailbox_capacity(MAILBOX_CAPACITY);

        self.subscribe_async::<BrokerType, FlowMessageWithMetadata>(ctx);
        self.subscribe_async::<BrokerType, AckMessage>(ctx);
        self.subscribe_async::<BrokerType, InitRetrier>(ctx);
        self.subscribe_async::<BrokerType, InitConsumer>(ctx);
        self.subscribe_async::<BrokerType, FlushCollectedEventsToPipeline>(ctx);

        tokio::spawn({
            let broker = self.broker.clone();
            async move {
                broker.lock().await.issue_async(InitConsumer);
                broker.lock().await.issue_async(InitRetrier);
            }
        });
        info!("[event_stream actor] Subscribed to desired kind of messages");
    }
}

impl<T, R> Handler<FlowMessageWithMetadata> for EventStreamActor<T, R>
where
    T: Transport + 'static,
    R: RetrierExt + 'static,
{
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: FlowMessageWithMetadata, _ctx: &mut Self::Context) -> Self::Result {
        debug!(target: "consumed_events", "Got message {}", msg.metadata.offset.unwrap());
        let broker = self.broker.clone();
        Box::pin(async move {
            broker
                .lock()
                .await
                .issue_async::<ClassifyFlowMessageWithMetadata>(msg.into());
        })
    }
}

impl<T, R> Handler<InitConsumer> for EventStreamActor<T, R>
where
    T: Transport + 'static,
    R: RetrierExt + 'static,
{
    type Result = ();

    fn handle(&mut self, _msg: InitConsumer, ctx: &mut Self::Context) -> Self::Result {
        info!("Invoked InitConsumer");
        let processor = self.processor.clone();
        let broker = self.broker.clone();
        let (tx, rx) = tokio::sync::mpsc::channel::<usize>(1);
        self.notify_channel = Some(tx);

        let actor_fut = Box::pin(async move {
            let guarding_fut = processor.guard_acks();
            let consuming_fut = processor.consume(broker.clone(), rx);

            broker
                .lock()
                .await
                .issue_async(FlushCollectedEventsToPipeline(MAILBOX_CAPACITY));
            let ((), ()) = tokio::join!(guarding_fut, consuming_fut);
        })
        .into_actor(self);

        ctx.spawn(actor_fut);
    }
}

impl<T, R> Handler<FlushCollectedEventsToPipeline> for EventStreamActor<T, R>
where
    T: Transport + 'static,
    R: RetrierExt + 'static,
{
    type Result = ResponseFuture<()>;

    fn handle(
        &mut self,
        msg: FlushCollectedEventsToPipeline,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let tx = self.notify_channel.clone();
        Box::pin(async move {
            match tx {
                Some(tx) => {
                    if let Err(e) = tx.send(msg.0).await {
                        error!("unable to send capacity to consumer loop: {}", e);
                    }
                }
                None => {
                    panic!("Missing notification channel for processing")
                }
            }
        })
    }
}

impl<T, R> Handler<InitRetrier> for EventStreamActor<T, R>
where
    T: Transport + 'static,
    R: RetrierExt + 'static,
{
    type Result = ResponseFuture<()>;

    fn handle(&mut self, _msg: InitRetrier, _ctx: &mut Self::Context) -> Self::Result {
        let retrier = self.retrier.clone();
        info!("Invoked InitRetrier");

        Box::pin(async move { retrier.spawn_retriers().await })
    }
}

impl<T, R> Handler<AckMessage> for EventStreamActor<T, R>
where
    T: Transport + 'static,
    R: RetrierExt + 'static,
{
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: AckMessage, _ctx: &mut Self::Context) -> Self::Result {
        let retrier = self.retrier.clone();
        let processor = self.processor.clone();

        Box::pin(async move {
            match msg {
                AckMessage::Ack(offset, partition) => {
                    processor.ack(offset, partition);
                }
                AckMessage::NackRetry(mut msg) => {
                    debug!(
                        "Failed to process message with id: {} after {} try",
                        msg.metadata.id, msg.metadata.retry
                    );
                    msg.metadata.retry += 1;

                    if let Some(t) = &retrier.get_topic_based_on_retry(msg.metadata.retry) {
                        processor
                            .produce(t, retrier.get_brokers_retry(), &msg)
                            .await;
                        let offst = msg.metadata.offset.unwrap();
                        processor.ack(offst, msg.metadata.partition.unwrap());
                    }
                }
            }
        })
    }
}
