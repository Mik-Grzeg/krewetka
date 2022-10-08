use async_trait::async_trait;
use crate::pb::{flow_message_classifier_client::FlowMessageClassifierClient, FlowMessage, FlowMessageClass};
use tonic::transport::Channel;
use futures::stream::Stream;
use tokio_stream::StreamExt;


use log::info;
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

// TODO temporary function, create tests from that
fn classifier_requests_iter() -> impl Stream<Item = FlowMessage> {
    tokio_stream::iter(1..usize::MAX).map(|i| FlowMessage {
        out_bytes: 1+i as u64,
        out_pkts: 2,
        in_bytes: 3,
        in_pkts: 4,
        ipv4_src_addr: "192.168.1.1".into(),
        ipv4_dst_addr: "192.168.1.2".into(),
        l7_proto: "test".into(),
        l4_dst_port: 5507,
        l4_src_port: 4091,
        flow_duration_milliseconds: 123,
        protocol: 7,
        tcp_flags: 11,
    })
}

pub async fn streaming_classifier(client: &mut FlowMessageClassifierClient<Channel>, num: usize) {
    let in_stream =  classifier_requests_iter().take(num);

    let response = client
        .classify_streaming(in_stream)
        .await
        .unwrap();

    let mut resp_stream = response.into_inner();

    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();
        info!("received message: `{}`", received.malicious);
    }
}