use std::str::Utf8Error;

#[derive(Debug)]
pub enum EventStreamError {
    UnableToParseKafkaHeader(String),
}

impl From<Utf8Error> for EventStreamError {
    fn from(e: Utf8Error) -> Self {
        EventStreamError::UnableToParseKafkaHeader(e.to_string())
    }
}
