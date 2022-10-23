use super::storage_actor::{AStorage, StorageError};
// use crate::actors::acknowleger::messages::PutOnRetryMessage;

use crate::actors::acker::messages::AckMessage;
use crate::actors::BrokerType;
use actix_broker::Broker;
use std::time::Duration;
use chrono::{TimeZone, Utc};
use clickhouse_rs::{row, types::Block, Pool};

use log::{error, info};
use serde::Deserialize;

use crate::actors::messages::FlowMessageWithMetadata;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Deserialize)]
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
}

#[async_trait]
impl AStorage for ClickhouseState {
    async fn stash(&self, msgs: Vec<FlowMessageWithMetadata>) -> Result<(), StorageError> {
        let handler = self
            .pool
            .as_ref()
            .get_handle();
            // .await
            // .map_err(|e| StorageError::Database(Box::new(e)))?;
        
        let mut client = match tokio::time::timeout(Duration::from_secs(3), handler).await {
            Ok(res) => res.map_err(|e| StorageError::Database(Box::new(e)))?,
            Err(e) => return Err(StorageError::Database(Box::new(e))),
        };

        let mut block = Block::with_capacity(msgs.len());
        for f in msgs.iter() {
            if let Err(_e) = block.push(row! {
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
               timestamp:      Utc.timestamp(f.metadata.timestamp as i64, 0),
            }) {
                // TODO
                // figure out how to handle different message with the same content
                // on top of that remove that clone
                Broker::<BrokerType>::issue_async(AckMessage::NackRetry(f.to_owned()));
            }
        }

        match client.insert("messages", block).await {
            Ok(()) => {
                info!("saved {} messages to storage", msgs.len());
                msgs.iter().for_each(|m| {
                    Broker::<BrokerType>::issue_async(AckMessage::Ack(m.metadata.offset.unwrap()))
                });
                Ok(())
            }
            Err(e) => {
                error!("unable to insert messages to clickhouse: {}", e);
                msgs.into_iter()
                    .for_each(|m| Broker::<BrokerType>::issue_async(AckMessage::NackRetry(m)));
                Err(e.into())
            }
        }
    }
}
