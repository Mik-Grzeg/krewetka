use crate::pb::FlowMessage;
use chrono::DateTime;
use chrono::Utc;

use actix::Message;

pub enum ProcessedFinished {
    Ack(i64),
    Nack(i64),
}

#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct MessageFromEventStream {
    pub msg: FlowMessageWithMetadata,
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct FlowMessageWithMetadata {
    pub flow_message: FlowMessage,
    pub timestamp: DateTime<Utc>,
    pub host: String,
    pub malicious: Option<bool>,
    pub id: i64,
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct ClassifyFlowMessageWithMetadata {
    pub flow_message: FlowMessage,
    pub timestamp: DateTime<Utc>,
    pub host: String,
    pub malicious: Option<bool>,
    pub id: i64,
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct PersistFlowMessageWithMetadata {
    pub flow_message: FlowMessage,
    pub timestamp: DateTime<Utc>,
    pub host: String,
    pub malicious: Option<bool>,
    pub id: i64,
}

impl From<FlowMessageWithMetadata> for ClassifyFlowMessageWithMetadata {
    fn from(original_flow_message: FlowMessageWithMetadata) -> Self {
        Self {
            flow_message: original_flow_message.flow_message,
            timestamp: original_flow_message.timestamp,
            host: original_flow_message.host,
            malicious: original_flow_message.malicious,
            id: original_flow_message.id,
        }
    }
}

impl From<ClassifyFlowMessageWithMetadata> for PersistFlowMessageWithMetadata {
    fn from(original_flow_message: ClassifyFlowMessageWithMetadata) -> Self {
        Self {
            flow_message: original_flow_message.flow_message,
            timestamp: original_flow_message.timestamp,
            host: original_flow_message.host,
            malicious: original_flow_message.malicious,
            id: original_flow_message.id,
        }
    }
}

impl From<PersistFlowMessageWithMetadata> for FlowMessageWithMetadata {
    fn from(original_flow_message: PersistFlowMessageWithMetadata) -> FlowMessageWithMetadata {
        Self {
            flow_message: original_flow_message.flow_message,
            timestamp: original_flow_message.timestamp,
            host: original_flow_message.host,
            malicious: original_flow_message.malicious,
            id: original_flow_message.id,
        }
    }
}
