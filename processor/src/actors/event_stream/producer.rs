use crate::actors::messages::FlowMessageWithMetadata;

pub trait Producer {
    fn produce(&self, msg: FlowMessageWithMetadata);
}
