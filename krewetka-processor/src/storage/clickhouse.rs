use clickhouse_rs::{Pool, types::{Block, Options}};
use super::astorage::{StorageError, AStorage};
use url::Url;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ClickhouseSettings {
    host: String,
    port: u16,
    user: String,
    password: String,
}

impl std::fmt::Display for ClickhouseSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "tcp://{}:{}@{}:{}/default?compression=lz4", self.user, self.password, self.host, self.port)
    }
}

pub struct ClickhouseState {
    pub settings: ClickhouseSettings,
    pub pool: Arc<Pool>,
}

impl ClickhouseState {
    pub fn new(host: String, port: u16, user: String, password: String) -> Self {
        let settings = ClickhouseSettings {
            host,
            port,
            user,
            password
        };
        let dsn = settings.to_string();


        let pool = Arc::new(Pool::new(dsn));

        Self {
            settings,
            pool,
        }
    }
}

#[async_trait]
impl AStorage for ClickhouseState {
    async fn stash(&self) -> Result<(), StorageError> {
        let mut client = self
            .pool
            .as_ref()
            .get_handle()
            .await
            .map_err(|e| StorageError::Database(Box::new(e)))?;


        Ok(())
    }
}
