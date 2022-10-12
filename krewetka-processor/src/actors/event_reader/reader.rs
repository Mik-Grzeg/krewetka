use crate::actors::messages::MessageInPipeline;

use crate::pb::FlowMessage;

use actix::ResponseFuture;

use actix::{Actor, Addr, Context, Handler};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use log::{info};
use rdkafka::message::OwnedMessage;


use std::sync::Arc;


use super::super::classification_client_grpc::client::ClassificationActor;
use super::super::messages::MessageFromEventStream;
use super::kafka::KafkaState;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn consume(&self, next: Addr<EventStreamReaderActor>);
    // async fn send_to_actor<S: Message + Send, T: Actor + Handler<S>>(&self, msg: OwnedMessage, next: Addr<T>);
    async fn send_to_actor(&self, msg: OwnedMessage, next: &Addr<EventStreamReaderActor>);
    async fn consume_batch(&self, tx: tokio::sync::broadcast::Sender<FlowMessageWithMetadata>);
}

#[derive(Clone, Debug)]
pub struct FlowMessageWithMetadata {
    pub flow_message: FlowMessage,
    pub timestamp: DateTime<Utc>,
    pub host: String,
    pub malicious: Option<bool>,
    // id:             Uuid,
}

pub struct EventStreamReaderActor {
    pub channel: Arc<KafkaState>,
    pub next: Addr<ClassificationActor>,
}

impl Actor for EventStreamReaderActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Started transport actor");
    }
}

impl Handler<MessageFromEventStream> for EventStreamReaderActor {
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: MessageFromEventStream, _ctx: &mut Self::Context) -> Self::Result {
        let next = self.next.clone();
        Box::pin(async move {
            match next.send(MessageInPipeline(msg.msg)).await {
                Err(_e) => Err(()),
                Ok(r) => Ok(r),
            }
        })
    }
}
