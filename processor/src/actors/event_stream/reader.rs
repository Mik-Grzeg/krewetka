use super::super::messages::FlowMessageWithMetadata;
use super::super::BrokerType;
use super::kafka::KafkaState;
use crate::actors::messages::ClassifyFlowMessageWithMetadata;
use actix::{Actor, Context, Handler};
use actix_broker::{BrokerIssue, BrokerSubscribe};
use async_trait::async_trait;
use log::info;
use std::sync::Arc;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn consume(&self);
    // async fn send_to_actor<S: Message + Send, T: Actor + Handler<S>>(&self, msg: OwnedMessage, next: Addr<T>);
}

pub struct EventStreamActor {
    pub channel: Arc<KafkaState>,
    // pub broker: Arc<Mutex<Broker>>,
}

impl Actor for EventStreamActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started transport actor");
        self.subscribe_async::<BrokerType, FlowMessageWithMetadata>(ctx);
    }
}

impl Handler<FlowMessageWithMetadata> for EventStreamActor {
    type Result = ();

    fn handle(&mut self, msg: FlowMessageWithMetadata, _ctx: &mut Self::Context) -> Self::Result {
        info!("Got message: {:?}", msg);
        self.issue_async::<BrokerType, ClassifyFlowMessageWithMetadata>(msg.into());
    }
}
