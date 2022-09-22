use super::astorage::{AStorage, StorageError};
use clickhouse_rs::Pool;
use serde::Deserialize;

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
    async fn stash(&self) -> Result<(), StorageError> {
        let _client = self
            .pool
            .as_ref()
            .get_handle()
            .await
            .map_err(|e| StorageError::Database(Box::new(e)))?;

        Ok(())
    }
}
