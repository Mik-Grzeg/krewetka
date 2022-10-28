use crate::pb::FlowMessage;

use super::event_stream::errors::EventStreamError;
use actix::Message;

pub enum ProcessedFinished {
    Ack(i64),
    Nack(i64),
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
#[derive(Clone)]
pub enum AckMessage {
    Ack(i64),
    NackRetry(FlowMessageWithMetadata),
}

#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct MessageFromEventStream(pub FlowMessageWithMetadata);

#[derive(Clone, Debug)]
pub struct FlowMessageMetadata {
    pub timestamp: u64,
    pub host: String,
    pub id: String,
    pub retry: usize,
    pub offset: Option<i64>,
}

// TODO move it to kafka dir
impl FlowMessageMetadata {
    pub fn map_hdr<T>(r: Option<(&str, T)>, hdr: &str) -> Result<T, EventStreamError> {
        match r {
            Some((_h, v)) => Ok(v),
            None => Err(EventStreamError::UnableToParseKafkaHeader(hdr.to_owned())),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct FlowMessageWithMetadata {
    pub flow_message: FlowMessage,
    pub malicious: Option<bool>,
    pub metadata: FlowMessageMetadata,
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct ClassifyFlowMessageWithMetadata(pub FlowMessageWithMetadata);

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct PersistFlowMessageWithMetadata(pub FlowMessageWithMetadata);

impl From<FlowMessageWithMetadata> for ClassifyFlowMessageWithMetadata {
    fn from(original_flow_message: FlowMessageWithMetadata) -> Self {
        Self(original_flow_message)
    }
}

impl From<ClassifyFlowMessageWithMetadata> for PersistFlowMessageWithMetadata {
    fn from(original_flow_message: ClassifyFlowMessageWithMetadata) -> Self {
        Self(original_flow_message.0)
    }
}

impl From<PersistFlowMessageWithMetadata> for FlowMessageWithMetadata {
    fn from(original_flow_message: PersistFlowMessageWithMetadata) -> FlowMessageWithMetadata {
        original_flow_message.0
    }
}
