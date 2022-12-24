use super::storage_actor::{AStorage, StorageError};
// use crate::actors::acknowleger::messages::PutOnRetryMessage;

use crate::actors::messages::AckMessage;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use clickhouse_rs::{row, types::Block, Pool};
use futures::stream::StreamExt;
use std::time::Duration;

use log::error;
use serde::Deserialize;

use crate::actors::messages::FlowMessageWithMetadata;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ClickhouseSettings {
    host: String,
    port: u16,
    user: String,
    password: String,
}

impl From<ClickhouseSettings> for ClickhouseState {
    fn from(settings: ClickhouseSettings) -> ClickhouseState {
        ClickhouseState::new(settings)
    }
}

impl std::fmt::Display for ClickhouseSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "tcp://{}:{}@{}:{}/default?compression=lz4&send_retries=0",
            self.user, self.password, self.host, self.port
        )
    }
}

pub struct ClickhouseState {
    pub settings: ClickhouseSettings,
    pub pool: Arc<Pool>,
}

impl ClickhouseState {
    pub fn new(settings: ClickhouseSettings) -> Self {
        let dsn = settings.to_string();
        let pool = Arc::new(Pool::new(dsn));

        Self { settings, pool }
    }

    fn push_to_block(block: &mut Block, f: &FlowMessageWithMetadata) -> AckMessage {
        let ts_secs = f.metadata.timestamp / 1000;
        let ts_ns = f.metadata.timestamp % 1000 * 1_000_000;

        match block.push(row! {
           host: f.metadata.host.as_str(),
           out_bytes: f.flow_message.out_bytes,
           out_pkts:   f.flow_message.out_pkts,
           in_bytes:   f.flow_message.in_bytes,
           in_pkts:    f.flow_message.in_pkts,
           ipv4_src_addr: f.flow_message.ipv4_src_addr.as_str(),
           ipv4_dst_addr: f.flow_message.ipv4_dst_addr.as_str(),
           l7_proto:       f.flow_message.l7_proto,
           l4_dst_port:    f.flow_message.l4_dst_port,
           l4_src_port:    f.flow_message.l4_src_port,
           flow_duration_milliseconds: f.flow_message.flow_duration_milliseconds,
           protocol:       f.flow_message.protocol,
           tcp_flags:      f.flow_message.tcp_flags,
           malicious:      f.malicious.unwrap_or(false),
           timestamp:      DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ts_secs as i64, ts_ns as u32), Utc)
        }) {
            Ok(()) => AckMessage::Ack(f.metadata.offset.unwrap(), f.metadata.partition.unwrap()),
            Err(_e) => AckMessage::NackRetry(f.to_owned()),
        }
    }
}

#[async_trait]
impl AStorage for ClickhouseState {
    async fn stash(
        &self,
        msgs: Vec<FlowMessageWithMetadata>,
    ) -> Result<Vec<AckMessage>, StorageError> {
        let handler = self.pool.as_ref().get_handle();

        let mut client = match tokio::time::timeout(Duration::from_secs(3), handler).await {
            Ok(res) => res.map_err(|e| {
                StorageError::DatabaseSave((
                    Box::new(e),
                    msgs.clone()
                        .into_iter()
                        .map(AckMessage::NackRetry)
                        .collect::<Vec<AckMessage>>(),
                ))
            })?,
            Err(e) => {
                return Err(StorageError::DatabaseSave((
                    Box::new(e),
                    msgs.into_iter()
                        .map(AckMessage::NackRetry)
                        .collect::<Vec<AckMessage>>(),
                )))
            }
        };

        let mut block = Block::with_capacity(msgs.len());

        let acks = msgs
            .iter()
            .map(|f| ClickhouseState::push_to_block(&mut block, f))
            .collect::<Vec<AckMessage>>();

        match client.insert("messages", block).await {
            Ok(()) => Ok(acks),
            Err(e) => {
                error!("unable to insert messages to clickhouse: {}", e);
                let nacks = msgs
                    .into_iter()
                    .map(AckMessage::NackRetry)
                    .collect::<Vec<AckMessage>>();
                Err(StorageError::DatabaseSave((Box::new(e), nacks)))
            }
        }
    }
}
