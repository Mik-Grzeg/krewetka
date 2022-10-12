use super::event_reader::FlowMessageWithMetadata;

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
