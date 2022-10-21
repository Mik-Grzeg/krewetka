use crate::actors::broker::Broker;

use crate::actors::messages::FlowMessageWithMetadata;

use async_trait::async_trait;

use std::sync::Arc;
use std::sync::Mutex;

#[async_trait]
pub trait Acknowleger {
    async fn end_processing(&self, id: i64);
    fn produce_on_retry(&self, msg: FlowMessageWithMetadata);
}

pub struct AckActor<A> {
    pub acker: Arc<A>,
    pub broker: Arc<Mutex<Broker>>,
}

// impl<A> Actor for AckActor<A>
// where
//     A: Acknowleger + 'static,
// {
//     type Context = Context<Self>;

//     fn started(&mut self, ctx: &mut Self::Context) {
//         info!("Started acking actor");
//         self.subscribe_async::<BrokerType, AckMessage>(ctx);
//     }
// }

// impl<A> Handler<AckMessage> for AckActor<A>
// where
//     A: Acknowleger + TopicOffsetKeeper + 'static,
// {
//     type Result = ResponseFuture<()>;

//     fn handle(&mut self, msg: AckMessage, _ctx: &mut Self::Context) -> Self::Result {
//         let acker = self.acker.clone();

//         Box::pin(async move {
//             match msg {
//                 AckMessage::Ack(offset, retry) => acker.stash_processed_offset(offset, retry),
//                 AckMessage::NackRetry(mut msg) => {
//                     debug!("Failed to process message with id: {} after {} try", msg.metadata.id, msg.metadata.retry);
//                     msg.metadata.retry += 1;
//                     acker.produce_on_retry(msg);
//                 }
//             }
//         })
//     }
// }
