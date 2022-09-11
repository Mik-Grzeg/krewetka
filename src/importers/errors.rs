#[derive(Debug, PartialEq)]
pub enum ImporterError {
    ZMQErr(zmq::Error),
}
