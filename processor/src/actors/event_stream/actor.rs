use super::super::BrokerType;
use super::RetrierExt;
use super::Transport;
use actix::{Actor, Context, Handler, ResponseFuture};
use actix_broker::{BrokerIssue, BrokerSubscribe};
use std::sync::Arc;

use super::messages::InitConsumer;
use super::messages::InitRetrier;
use crate::actors::messages::AckMessage;
use crate::actors::messages::ClassifyFlowMessageWithMetadata;
use crate::actors::messages::FlowMessageWithMetadata;
use log::*;

pub struct EventStreamActor<T, R> {
    pub processor: Arc<T>,
    pub retrier: Arc<R>,
}

impl<T, R> Actor for EventStreamActor<T, R>
where
    T: Transport + 'static + Unpin,
    R: RetrierExt + 'static + Unpin,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started transport actor");
        self.subscribe_async::<BrokerType, FlowMessageWithMetadata>(ctx);
        self.subscribe_async::<BrokerType, AckMessage>(ctx);
        self.subscribe_async::<BrokerType, InitRetrier>(ctx);
        self.subscribe_async::<BrokerType, InitConsumer>(ctx);
    }
}

impl<T, R> EventStreamActor<T, R>
where
    T: Transport + 'static + Unpin,
    R: RetrierExt + 'static + Unpin,
{
    pub async fn start_streaming(&self) {
        let processor = self.processor.clone();
        let retrier = self.retrier.clone();

        tokio::join!(processor.consume(), retrier.spawn_retriers(),);
    }
}

impl<T, R> Handler<FlowMessageWithMetadata> for EventStreamActor<T, R>
where
    T: Transport + 'static + Unpin,
    R: RetrierExt + 'static + Unpin,
{
    type Result = ();

    fn handle(&mut self, msg: FlowMessageWithMetadata, _ctx: &mut Self::Context) -> Self::Result {
        self.issue_async::<BrokerType, ClassifyFlowMessageWithMetadata>(msg.into());
    }
}

impl<T, R> Handler<InitConsumer> for EventStreamActor<T, R>
where
    T: Transport + 'static + Unpin,
    R: RetrierExt + 'static + Unpin,
{
    type Result = ResponseFuture<()>;

    fn handle(&mut self, _msg: InitConsumer, _ctx: &mut Self::Context) -> Self::Result {
        let processor = self.processor.clone();

        Box::pin(async move {
            let consuming_fut = processor.consume();
            let guarding_fut = processor.guard_acks();

            let ((), ()) = tokio::join!(consuming_fut, guarding_fut);
        })
    }
}

impl<T, R> Handler<InitRetrier> for EventStreamActor<T, R>
where
    T: Transport + 'static + Unpin,
    R: RetrierExt + 'static + Unpin,
{
    type Result = ResponseFuture<()>;

    fn handle(&mut self, _msg: InitRetrier, _ctx: &mut Self::Context) -> Self::Result {
        let retrier = self.retrier.clone();

        Box::pin(async move { retrier.spawn_retriers().await })
    }
}

impl<T, R> Handler<AckMessage> for EventStreamActor<T, R>
where
    T: Transport + 'static + Unpin,
    R: RetrierExt + 'static + Unpin,
{
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: AckMessage, _ctx: &mut Self::Context) -> Self::Result {
        let retrier = self.retrier.clone();
        let processor = self.processor.clone();

        Box::pin(async move {
            match msg {
                AckMessage::Ack(offset) => {
                    processor.ack(offset);
                }
                AckMessage::NackRetry(mut msg) => {
                    warn!(
                        "Failed to process message with id: {} after {} try",
                        msg.metadata.id, msg.metadata.retry
                    );
                    msg.metadata.retry += 1;

                    if let Some(t) = &retrier.get_topic_based_on_retry(msg.metadata.retry) {
                        processor
                            .produce(t, retrier.get_brokers_retry(), &msg)
                            .await;
                        let offst = msg.metadata.offset.unwrap();
                        info!("Offset to nackretry: {}", offst);
                        processor.ack(offst);
                    }
                }
            }
        })
    }
}
