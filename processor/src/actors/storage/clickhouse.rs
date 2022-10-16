use super::storage_actor::{AStorage, StorageError};
// use crate::actors::acknowleger::messages::PutOnRetryMessage;

use crate::actors::acker::messages::AckMessage;
use crate::actors::BrokerType;
use actix_broker::Broker;

use clickhouse_rs::{row, types::Block, Pool};

use log::info;
use serde::Deserialize;

use crate::actors::messages::{FlowMessageWithMetadata, PersistFlowMessageWithMetadata};
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
            "tcp://{}:{}@{}:{}/default?compression=lz4",
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
    async fn stash(&self, msgs: Vec<PersistFlowMessageWithMetadata>) -> Result<(), StorageError> {
        let mut client = self
            .pool
            .as_ref()
            .get_handle()
            .await
            .map_err(|e| StorageError::Database(Box::new(e)))?;

        let mut block = Block::with_capacity(msgs.len());
        for f in msgs.iter() {
            if let Err(_e) = block.push(row! {
               host: f.host.as_str(),
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
               timestamp:      f.timestamp
            }) {
                // TODO
                // figure out how to handle different message with the same content
                // on top of that remove that clone
                Broker::<BrokerType>::issue_async(AckMessage::NackRetry(
                    FlowMessageWithMetadata::from(f.clone()),
                ));
            }
        }

        info!("Saving to clickhouse {} messages", msgs.len());

        match client.insert("messages", block).await {
            Ok(()) => {
                msgs.iter()
                    .for_each(|m| Broker::<BrokerType>::issue_async(AckMessage::Ack(m.id)));
                Ok(())
            }
            Err(e) => {
                msgs.into_iter().for_each(|m| {
                    Broker::<BrokerType>::issue_async(AckMessage::NackRetry(
                        FlowMessageWithMetadata::from(m),
                    ))
                });
                Err(e.into())
            }
        }
    }
}
