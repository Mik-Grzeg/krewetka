use super::super::errors::EventStreamError;
use crate::actors::messages::FlowMessageMetadata;
use rdkafka::message::FromBytes;
use rdkafka::message::Headers;
use rdkafka::message::{BorrowedHeaders, OwnedHeaders};

impl TryFrom<&OwnedHeaders> for FlowMessageMetadata {
    type Error = EventStreamError;

    fn try_from(headers: &OwnedHeaders) -> Result<Self, Self::Error> {
        let host = str::from_bytes(FlowMessageMetadata::map_hdr(
            headers.get(0),
            "host-identifier-x",
        )?)?
        .to_owned();
        let id = str::from_bytes(FlowMessageMetadata::map_hdr(
            headers.get(1),
            "message-id-x",
        )?)?
        .to_owned();
        let retry = str::from_bytes(FlowMessageMetadata::map_hdr(headers.get(3), "retry-x")?)?
            .to_owned()
            .parse::<usize>()
            .unwrap();
        let timestamp =
            str::from_bytes(FlowMessageMetadata::map_hdr(headers.get(2), "timestamp-x")?)?
                .to_owned()
                .parse::<u64>()
                .unwrap();

        Ok(FlowMessageMetadata {
            host,
            id,
            timestamp,
            retry,
            offset: None,
            partition: None,
        })
    }
}

impl TryFrom<&BorrowedHeaders> for FlowMessageMetadata {
    type Error = EventStreamError;

    fn try_from(headers: &BorrowedHeaders) -> Result<Self, Self::Error> {
        let host = str::from_bytes(FlowMessageMetadata::map_hdr(
            headers.get(0),
            "host-identifier-x",
        )?)?
        .to_owned();
        let id = str::from_bytes(FlowMessageMetadata::map_hdr(
            headers.get(1),
            "message-id-x",
        )?)?
        .to_owned();
        let retry = str::from_bytes(FlowMessageMetadata::map_hdr(headers.get(3), "retry-x")?)?
            .to_owned()
            .parse::<usize>()
            .unwrap();
        let timestamp =
            str::from_bytes(FlowMessageMetadata::map_hdr(headers.get(2), "timestamp-x")?)?
                .to_owned()
                .parse::<u64>()
                .unwrap();

        Ok(FlowMessageMetadata {
            host,
            id,
            timestamp,
            retry,
            offset: None,
            partition: None,
        })
    }
}
