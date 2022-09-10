use core::fmt;

use std::sync::Arc;

use zmq::{Context, Socket};
use log::{info, debug};

use async_trait::async_trait;

use super::{import::Import, errors::ImporterError};

#[derive(Debug)]
pub struct ZMQSettings {
    pub address: String,
    pub queue_name: String
}

pub struct ZMQ {
    pub subscriber: Socket,
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
        info!("successfuly connected to socket at: [{}]", subscriber_connection);

        let zmq_queue = settings.queue_name.as_bytes();

        subscriber
            .set_subscribe(zmq_queue)
            .expect("Failed setting subscription");
        info!("successfuly subscribed to zmq queue: [{}]", settings.queue_name);

        ZMQ {
            settings: settings,
            subscriber: subscriber,
        }
    }
}

unsafe impl Send for ZMQ {}
unsafe impl Sync for ZMQ {}

#[async_trait]
impl Import for ZMQ {
    async fn import(&self) -> Result<Vec<u8>, ImporterError> {
        let mut messages = self.subscriber
            .recv_multipart(0)
            .map_err(ImporterError::ZMQErr)?;
        let msg = messages.remove(1);

        debug!("Imported message: {:?}", String::from_utf8_lossy(&msg));
        Ok(msg)
    }
}
