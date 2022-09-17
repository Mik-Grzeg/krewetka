#[derive(Debug)]
pub enum ImporterError {
    ZMQErr(zmq::Error),
    DeserializationErr(serde_json::Error),
}

impl PartialEq for ImporterError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ZMQErr(a), Self::ZMQErr(b)) => a.eq(b),
            (Self::DeserializationErr(a), Self::DeserializationErr(b)) => {
                a.classify() == b.classify()
            }
            _ => false,
        }
    }
}

impl From<serde_json::Error> for ImporterError {
    fn from(error: serde_json::Error) -> Self {
        Self::DeserializationErr(error)
    }
}
