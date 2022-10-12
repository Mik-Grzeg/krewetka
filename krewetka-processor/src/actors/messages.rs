use actix::Message;
use std::error::Error;
use super::event_reader::FlowMessageWithMetadata;
use actix::MailboxError;

pub enum ProcessedFinished {
    Ack,
    Nack
}

#[derive(Message)]
#[rtype(result = "Result<(), tonic::Code>")]
pub struct MessageToClassify {
    pub msg: FlowMessageWithMetadata
}

#[derive(Message)]
#[rtype(result = "Result<(), MailboxError>")]
pub struct MessageFromEventStream {
    pub msg: FlowMessageWithMetadata
}
