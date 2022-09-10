use super::errors::ImporterError;
use async_trait::async_trait;


#[async_trait]
pub trait Import: Sync + Send {
    async fn import(&self) -> Result<Vec<u8>, ImporterError>;
}
