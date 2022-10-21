use super::super::BrokerType;
use super::kafka::ProcessingAgent;
use super::{super::messages::FlowMessageWithMetadata, retrier::Retrier};
use crate::actors::acker::messages::AckMessage;
use crate::actors::messages::ClassifyFlowMessageWithMetadata;
use actix::{Actor, Context, Handler, ResponseFuture};
use actix_broker::{BrokerIssue, BrokerSubscribe};
use async_trait::async_trait;
use log::{info, warn};
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::Mutex;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn consume(&self, topic: String, brokers: String);
    async fn produce(&self, topic: &str, brokers: &str, msg: &FlowMessageWithMetadata);
    // async
    // async fn send_to_actor<S: Message + Send, T: Actor + Handler<S>>(&self, msg: OwnedMessage, next: Addr<T>);
}

pub struct EventStreamActor {
    pub processor: Arc<ProcessingAgent>,
    pub retrier: Arc<Retrier>,
    pub offset_heap: Arc<Mutex<BinaryHeap<i64>>>,
    // pub broker: Arc<Mutex<Broker>>,
}

impl Actor for EventStreamActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started transport actor");
        self.subscribe_async::<BrokerType, FlowMessageWithMetadata>(ctx);
        self.subscribe_async::<BrokerType, AckMessage>(ctx);
    }
}

impl Handler<FlowMessageWithMetadata> for EventStreamActor {
    type Result = ();

    fn handle(&mut self, msg: FlowMessageWithMetadata, _ctx: &mut Self::Context) -> Self::Result {
        self.issue_async::<BrokerType, ClassifyFlowMessageWithMetadata>(msg.into());
    }
}

impl Handler<AckMessage> for EventStreamActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: AckMessage, _ctx: &mut Self::Context) -> Self::Result {
        let retrier = self.retrier.clone();
        let processor = self.processor.clone();
        let offset_guard = self.offset_heap.clone();

        Box::pin(async move {
            match msg {
                AckMessage::Ack(offset) => offset_guard.lock().unwrap().push(offset),
                AckMessage::NackRetry(mut msg) => {
                    warn!(
                        "Failed to process message with id: {} after {} try",
                        msg.metadata.id, msg.metadata.retry
                    );
                    msg.metadata.retry += 1;

                    processor
                        .produce(
                            &retrier
                                .get_topic_based_on_retry(msg.metadata.retry)
                                .unwrap(),
                            retrier.get_brokers_retry(),
                            &msg,
                        )
                        .await;
                }
            }
        })
    }
}
