use crate::{
    actors::messages::{
        FlowMessageWithMetadata, PersistFlowMessageWithMetadata, ProcessedFinished,
    },
    pb::FlowMessage,
};
use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone)]
pub enum AckMessage {
    Ack(i64),
    NackRetry(FlowMessageWithMetadata),
}
