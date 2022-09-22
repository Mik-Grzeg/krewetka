use crate::flow::FlowMessage;
use async_trait::async_trait;

use std::sync::{Arc, Mutex};

#[async_trait]
pub trait Transport: Send + Sync {
    async fn consume(&self, tx: tokio::sync::mpsc::Sender<FlowMessage>);
    async fn consume_batch(&self, tx: tokio::sync::mpsc::Sender<WrappedFlowMessage>);
}

#[derive(Clone)]
pub struct WrappedFlowMessage(pub Arc<Mutex<FlowMessage>>);

// impl FromBytes for WrappedFlowMessage {
//     type Error = DecodeError;
//     fn from_bytes(b: &[u8]) -> Result<&Self, Self::Error> {
//         let res = PBMessage::decode::<&[u8]>(b)?;
//         let res1 = Self(Arc::new(Mutex::new(res)) );
//         Ok(&res1)
//     }
// }
