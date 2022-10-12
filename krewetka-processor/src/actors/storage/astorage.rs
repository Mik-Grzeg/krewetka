use crate::actors::event_reader::FlowMessageWithMetadata;
use actix::Actor;
use actix::Context;
use async_trait::async_trait;
use clickhouse_rs::errors::Error as ChError;
use std::error::Error;

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

pub struct StorageActor {}

impl Actor for StorageActor {
    type Context = Context<Self>;
}
