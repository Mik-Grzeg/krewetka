use crate::pb::FlowMessage;
use chrono::DateTime;
use chrono::Utc;

use actix::Message;

pub enum ProcessedFinished {
    Ack,
    Nack,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct MessageInPipeline(pub FlowMessageWithMetadata);

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
    // id:             Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct ClassifyFlowMessageWithMetadata {
    pub flow_message: FlowMessage,
    pub timestamp: DateTime<Utc>,
    pub host: String,
    pub malicious: Option<bool>,
    // id:             Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Clone, Debug)]
pub struct PersistFlowMessageWithMetadata {
    pub flow_message: FlowMessage,
    pub timestamp: DateTime<Utc>,
    pub host: String,
    pub malicious: Option<bool>,
    // id:             Uuid,
}

impl From<FlowMessageWithMetadata> for ClassifyFlowMessageWithMetadata {
    fn from(original_flow_message: FlowMessageWithMetadata) -> Self {
        Self {
            flow_message: original_flow_message.flow_message,
            timestamp: original_flow_message.timestamp,
            host: original_flow_message.host,
            malicious: original_flow_message.malicious,
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
        }
    }
}
