use async_trait::async_trait;
use std::error::Error;
use crate::{pb::FlowMessage, transport::FlowMessageWithMetadata};
use clickhouse_rs::errors::Error as ChError;

#[derive(Debug)]
pub enum StorageError {
    Database(Box<dyn Error>),
}

impl From<ChError> for StorageError {
    fn from(e: ChError) -> StorageError {
        StorageError::Database(Box::new(e))
    }
}

#[async_trait]
pub trait AStorage {
    async fn stash(&self, msgs: &Vec<FlowMessageWithMetadata>) -> Result<(), StorageError>;
}
