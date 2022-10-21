use crate::actors::broker::Broker;

use crate::actors::messages::{ClassifyFlowMessageWithMetadata, PersistFlowMessageWithMetadata};
use crate::actors::BrokerType;
use crate::{
    actors::messages::FlowMessageWithMetadata,
    pb::{flow_message_classifier_client::FlowMessageClassifierClient, FlowMessage},
};
use actix::ResponseFuture;
use std::sync::Arc;
use std::sync::Mutex;

use actix::Actor;
use actix::Context;
use actix::Handler;
use actix_broker::{BrokerIssue, BrokerSubscribe};
use log::error;
use tokio_stream::{Stream, StreamExt};
use tonic::transport::Channel;

use log::info;

pub struct ClassificationActor {
    pub client: FlowMessageClassifierClient<Channel>,
    pub broker: Arc<Mutex<Broker>>,
}

impl Actor for ClassificationActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Started  classification actor");
        self.subscribe_async::<BrokerType, ClassifyFlowMessageWithMetadata>(ctx)
    }
}

impl Handler<ClassifyFlowMessageWithMetadata> for ClassificationActor {
    type Result = ResponseFuture<()>;

    fn handle(
        &mut self,
        msg: ClassifyFlowMessageWithMetadata,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let mut client = self.client.clone();
        let mut msg = msg;
        let broker = self.broker.clone();

        Box::pin(async move {
            match client.classify(msg.0.flow_message.clone()).await {
                Ok(b) => {
                    msg.0.malicious = Some(b.get_ref().malicious);
                    broker
                        .lock()
                        .unwrap()
                        .issue_async::<PersistFlowMessageWithMetadata>(msg.into());
                }
                Err(e) => {
                    error!("Classify response: {:?}", e);
                }
            }
        })
    }
}

pub struct Classifier {
    pub host: String,
    pub port: u16,
}

pub fn classifier_requests_iter() -> impl Stream<Item = FlowMessage> {
    tokio_stream::iter(1..usize::MAX).map(|i| FlowMessage {
        out_bytes: 1 + i as u64,
        out_pkts: 2,
        in_bytes: 3,
        in_pkts: 4,
        ipv4_src_addr: "192.168.1.1".into(),
        ipv4_dst_addr: "192.168.1.2".into(),
        l7_proto: 7.2,
        l4_dst_port: 5507,
        l4_src_port: 4091,
        flow_duration_milliseconds: 123,
        protocol: 7,
        tcp_flags: 11,
    })
}

// pub async fn batched_classifier()

pub async fn streaming_classifier(
    rx: &mut tokio::sync::broadcast::Receiver<FlowMessageWithMetadata>,
    in_stream: tokio_stream::wrappers::BroadcastStream<FlowMessageWithMetadata>,
    tx_result: tokio::sync::mpsc::Sender<FlowMessageWithMetadata>,
    client: &mut FlowMessageClassifierClient<Channel>,
) {
    let in_stream = in_stream.map(|m| m.unwrap().flow_message);

    let response = client.classify_streaming(in_stream).await.unwrap();

    let mut resp_stream = response.into_inner();

    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();

        let mut message = rx.recv().await.unwrap();
        message.malicious = Some(received.malicious);

        tx_result.send(message).await.unwrap();
    }
}
