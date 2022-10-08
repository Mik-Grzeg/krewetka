use crate::{pb::{flow_message_classifier_client::FlowMessageClassifierClient, FlowMessage, FlowMessageClass}, transport::FlowMessageWithMetadata};
use tonic::transport::Channel;
// use futures::stream::Stream;
use tokio_stream::{StreamExt, StreamMap, Stream};
use std::pin::Pin;
use log::info;

pub struct Classifier {
    pub host: String,
    pub port: u16,
}

pub fn classifier_requests_iter() -> impl Stream<Item = FlowMessage> {
    tokio_stream::iter(1..usize::MAX).map(|i| FlowMessage {
        out_bytes: 1+i as u64,
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

pub async fn streaming_classifier(
    rx: &mut tokio::sync::broadcast::Receiver<FlowMessageWithMetadata>,
    in_stream: tokio_stream::wrappers::BroadcastStream<FlowMessageWithMetadata>,
    tx_result: tokio::sync::mpsc::Sender<FlowMessageWithMetadata>,
    client: &mut FlowMessageClassifierClient<Channel>
) {

    let in_stream = in_stream
        .map(|m| m.unwrap().flow_message);

    let response = client
        .classify_streaming(in_stream)
        .await
        .unwrap();

    let mut resp_stream = response.into_inner();

    while let Some(received) = resp_stream.next().await {
        let received = received.unwrap();
        info!("received message: `{}`", received.malicious);

        let mut message = rx.recv().await.unwrap();
        message.malicious = Some(received.malicious);

        tx_result.send(message).await.unwrap();
    }
}