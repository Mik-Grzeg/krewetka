use crate::pb::FlowMessage;
use actix::MailboxError;
use actix::WrapFuture;
use actix::ResponseFuture;
use actix::{Actor, Context, Handler, Message, ResponseActFuture, StreamHandler, Addr};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::stream::Stream;
use rdkafka::message::OwnedMessage;
use std::sync::Mutex;
use std::sync::Arc;
use std::error::Error;
use log::{info, debug, error};
use std::pin::Pin;
use crate::consts::ACTORS_MAILBOX_CAPACITY;
use super::super::messages::MessageToClassify;

use super::kafka::KafkaState;
use super::super::messages::MessageFromEventStream;
use super::super::classification_client_grpc::client::ClassificationActor;

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
    pub channel: Arc::<KafkaState>,
    pub next: Addr<ClassificationActor>
}

impl Actor for EventStreamReaderActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started transport actor");
    }
}

impl Handler<MessageFromEventStream> for EventStreamReaderActor
{
    type Result = ResponseFuture<Result<(), MailboxError>>;

    fn handle(&mut self, msg: MessageFromEventStream, ctx: &mut Self::Context) -> Self::Result {
        debug!("Handling message: {:?}", msg.msg);

        let next = self.next.clone();
        Box::pin(async move {
            match next.send(MessageToClassify{ msg: msg.msg }).await {
                Ok(res) => {
                    debug!("Ok result: {:?}", res.unwrap());
                    Ok(())
                }
                Err(err) => {
                    debug!("Err result: {}", err);
                    Err(err)
                }
            }
        })
    }
}
