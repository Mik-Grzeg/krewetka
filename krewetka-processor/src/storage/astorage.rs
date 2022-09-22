use std::error::Error;
use async_trait::async_trait;

#[derive(Debug)]
pub enum StorageError {
    Database(Box<dyn Error>),
}

#[async_trait]
pub trait AStorage {
    async fn stash(&self) -> Result<(), StorageError>;
}
