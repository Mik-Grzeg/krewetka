use super::messages::AckMessage;
use crate::actors::broker::Broker;
use crate::actors::event_stream::TopicOffsetKeeper;
use crate::actors::BrokerType;
use actix::Actor;
use actix::Context;
use actix::Handler;
use actix::ResponseFuture;
use actix_broker::BrokerSubscribe;

use async_trait::async_trait;
use log::debug;
use log::info;
use std::sync::Arc;
use std::sync::Mutex;

#[async_trait]
pub trait Acknowleger {
    async fn end_processing(&self, id: i64);
}

pub struct AckActor<A> {
    pub acker: Arc<Mutex<A>>,
    pub broker: Arc<Mutex<Broker>>,
}

impl<A> Actor for AckActor<A>
where
    A: Acknowleger + TopicOffsetKeeper + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started acking actor");
        self.subscribe_async::<BrokerType, AckMessage>(ctx);
    }
}

impl<A> Handler<AckMessage> for AckActor<A>
where
    A: Acknowleger + TopicOffsetKeeper + 'static,
{
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: AckMessage, _ctx: &mut Self::Context) -> Self::Result {
        let acker = self.acker.clone();

        Box::pin(async move {
            match msg {
                AckMessage::Ack(offset) => acker.lock().unwrap().stash_processed_offset(offset),
                AckMessage::NackRetry(msg) => {
                    debug!("Failed to process message with id: {}", msg.id);
                    todo!()
                }
            }
        })
    }
}
