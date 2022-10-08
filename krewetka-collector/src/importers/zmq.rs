use core::fmt;

use log::{debug, info};

use zmq::Socket;

use async_trait::async_trait;

use super::{
    errors::ImporterError,
    import::{Import, Subscriber},
};

use crate::flow::FlowMessage;

#[derive(Debug)]
pub struct ZMQSettings {
    pub address: String,
    pub queue_name: String,
}

struct MySubscriber(Socket);

impl Subscriber for MySubscriber {
    fn recv(&self) -> Result<Vec<u8>, ImporterError> {
        let mut messages = self.0.recv_multipart(0).map_err(ImporterError::ZMQErr)?;
        let msg = messages.remove(1);

        Ok(msg)
    }
}

pub struct ZMQ {
    pub subscriber: Box<dyn Subscriber>,
    pub settings: ZMQSettings,
}

impl fmt::Debug for ZMQ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.settings)
    }
}

impl ZMQ {
    pub fn new(settings: ZMQSettings) -> Self {
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();

        let subscriber_connection = format!("tcp://{}", settings.address);

        subscriber
            .connect(&subscriber_connection)
            .expect("Failed connecting subscriber");
        info!(
            "successfuly connected to socket at: [{}]",
            subscriber_connection
        );

        let zmq_queue = settings.queue_name.as_bytes();

        subscriber
            .set_subscribe(zmq_queue)
            .expect("Failed setting subscription");
        info!(
            "successfuly subscribed to zmq queue: [{}]",
            settings.queue_name
        );

        let boxed_subscriber = Box::new(MySubscriber(subscriber));
        ZMQ {
            settings,
            subscriber: boxed_subscriber,
        }
    }
}

unsafe impl Send for ZMQ {}
unsafe impl Sync for ZMQ {}

#[async_trait]
impl Import for ZMQ {
    async fn import(&self) -> Result<FlowMessage, ImporterError> {
        // instead of using nprobe there might be our collector
        // which will deserialize packets into netflow format flow message
        let received_slice = &self.subscriber.recv()?;

        debug!(
            "String message: {}",
            std::str::from_utf8(received_slice).unwrap()
        ); // TODO remove that
        let msg: FlowMessage =
            serde_json::from_slice(received_slice).map_err(ImporterError::DeserializationErr)?;

        debug!("Imported message: {:#?}", msg); // TODO remove that
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::flow::FlowMessage;
    use mockall::mock;
    use pretty_assertions::assert_eq;
    use serde_json::error::Category;
    use test_case::case;
    use tokio_test::block_on;

    type RecvResult = Result<Vec<u8>, ImporterError>;
    mock! {
        pub Socket { }

        impl Subscriber for Socket {
            fn recv(&self) -> RecvResult;
        }
    }

    #[case(77, 1, 53, "10.0.0.1".to_string(), "10.0.0.2".to_string(), 17, 56341, 61, 1, 0.2, 0, 12, None; "ensure correct input is deserialized as it should, with no errors")]
    #[case(77, 1, 53, "".to_string(), "".to_string(), 17, 56341, 61, 1, 3.2, 0, 12, Some(Category::Data); "ensure partially missing data results in error")]
    fn test_import_zmq(
        out_bytes: u64,
        out_pkts: u64,
        l4_dst_port: u32,
        ipv4_dst_addr: String,
        ipv4_src_addr: String,
        protocol: u32,
        l4_src_port: u32,
        in_bytes: u64,
        in_pkts: u64,
        l7_proto: f32,
        tcp_flags: u32,
        flow_duration_milliseconds: u64,
        de_error_category: Option<serde_json::error::Category>,
    ) {
        let prepared_msg = format!(
            r#"{{"OUT_BYTES":{},"OUT_PKTS":{},"L4_DST_PORT":{},"IPV4_DST_ADDR":"{}","IPV4_SRC_ADDR":"{}","PROTOCOL":{},"L4_SRC_PORT":{},"IN_BYTES":{},"IN_PKTS":{},"L7_PROTO":"{}","TCP_FLAGS":{},"FLOW_DURATION_MILLISECONDS":{}}}"#,
            out_bytes,
            out_pkts,
            l4_dst_port,
            ipv4_dst_addr,
            ipv4_src_addr,
            protocol,
            l4_src_port,
            in_bytes,
            in_pkts,
            l7_proto,
            tcp_flags,
            flow_duration_milliseconds
        );

        let mut socket = MockSocket::new();
        socket
            .expect_recv()
            .returning(move || Ok(prepared_msg.clone().into_bytes()));

        let settings = ZMQSettings {
            address: "localhost:5561".to_string(),
            queue_name: "flow".to_string(),
        };

        let zmq = ZMQ {
            subscriber: Box::new(socket),
            settings,
        };

        let flow_msg = FlowMessage {
            out_bytes,
            out_pkts,
            in_bytes,
            in_pkts,
            l7_proto,
            protocol,
            l4_dst_port,
            ipv4_dst_addr,
            ipv4_src_addr,
            tcp_flags,
            flow_duration_milliseconds,
        };

        let result = block_on(zmq.import());

        match result {
            Ok(o) => assert_eq!(o, flow_msg),
            Err(e) => match e {
                ImporterError::DeserializationErr(d) => {
                    assert_eq!(Some(d.classify()), de_error_category)
                }
                ImporterError::ZMQErr(z) => {
                    assert_eq!(None, de_error_category);
                    panic!("Shouldn't be here: {}", z)
                }
            },
        }
    }
}
