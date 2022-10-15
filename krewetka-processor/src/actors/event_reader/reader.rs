use crate::actors::messages::ClassifyFlowMessageWithMetadata;

use super::super::messages::FlowMessageWithMetadata;

use actix::{Actor, Context, Handler};
use actix_broker::{BrokerIssue, BrokerSubscribe};
use async_trait::async_trait;

use log::info;

use super::super::BrokerType;

use std::sync::Arc;

use super::kafka::KafkaState;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn consume(&self);
    // async fn send_to_actor<S: Message + Send, T: Actor + Handler<S>>(&self, msg: OwnedMessage, next: Addr<T>);
}

pub struct EventStreamReaderActor {
    pub channel: Arc<KafkaState>,
    // pub broker: Arc<Mutex<Broker>>,
}

impl Actor for EventStreamReaderActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started transport actor");
        self.subscribe_async::<BrokerType, FlowMessageWithMetadata>(ctx);
    }
}

impl Handler<FlowMessageWithMetadata> for EventStreamReaderActor {
    type Result = ();

    fn handle(&mut self, msg: FlowMessageWithMetadata, _ctx: &mut Self::Context) -> Self::Result {
        info!("Got message: {:?}", msg);
        self.issue_async::<BrokerType, ClassifyFlowMessageWithMetadata>(msg.into());
    }
}
