use core::fmt;

use log::{debug, info};
use zmq::{Socket};

use async_trait::async_trait;

use super::{errors::ImporterError, import::{Import, Subscriber}};

#[derive(Debug)]
pub struct ZMQSettings {
    pub address: String,
    pub queue_name: String,
}

struct MySubscriber(Socket);


impl Subscriber for MySubscriber {
    fn recv(&self) -> Result<Vec<u8>, ImporterError> {
        let mut messages = self
            .0
            .recv_multipart(0)
            .map_err(ImporterError::ZMQErr)?;
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
            settings: settings,
            subscriber: boxed_subscriber,
        }
    }
}

unsafe impl Send for ZMQ {}
unsafe impl Sync for ZMQ {}

#[async_trait]
impl Import for ZMQ {
    async fn import(&self) -> Result<Vec<u8>, ImporterError> {
        let msg = self.subscriber
            .recv()?;

        debug!("Imported message: {:?}", String::from_utf8_lossy(&msg));
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zmq::{Socket};
    use pretty_assertions::assert_eq;
    use mockall::mock;
    use tokio_test::{block_on, assert_err};
    use test_case::case;

    mock!{
        pub Socket { }

        impl Subscriber for Socket {
            fn recv(&self) -> Result<Vec<u8>, ImporterError>;
        }
    }

    #[case(b"test second part")]
    fn test_import_zmq(byte_str_input: &'static [u8]) {

        let mut socket = MockSocket::new();
        socket.expect_recv()
            .returning(move || Ok(byte_str_input.clone().to_vec()));


        let settings = ZMQSettings {
            address: "localhost:5561".to_string(),
            queue_name: "test".to_string(),
        };

        let zmq = ZMQ {
            subscriber: Box::new(socket),
            settings: settings,
        };

        let result = block_on(zmq.import());
        assert_eq!(result.as_ref().err(), None);
        assert_eq!(result.unwrap(), byte_str_input);
    }
}

