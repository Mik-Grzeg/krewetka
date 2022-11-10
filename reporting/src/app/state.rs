use super::config::{AppConfigCache, Settings};
use super::errors::ConfigErr;
use clickhouse_rs::Pool;

use log::info;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct State {
    pub cfg_cache: AppConfigCache,
    pub db: Arc<Pool>,
}

fn init_db(host: &str, port: u16, user: &str, password: &str) -> Pool {
    let dsn = format!(
        "tcp://{}:{}@{}:{}/default?compression=lz4&send_retries=0",
        user, password, host, port
    );
    info!("Db dsn: {dsn}");
    Pool::new(dsn)
}

impl State {
    pub fn config(&self) -> Result<Settings, ConfigErr> {
        self.cfg_cache.get_config::<Settings>()
    }

    pub async fn new() -> Result<Self, ConfigErr> {
        let config_cache = AppConfigCache::new()?;
        let cfg = config_cache.get_config::<Settings>()?;

        let db_pool = Arc::new(init_db(
            &cfg.clickhouse_host,
            cfg.clickhouse_port,
            &cfg.clickhouse_user,
            &cfg.clickhouse_password,
        ));

        Ok(Self {
            db: db_pool,
            cfg_cache: config_cache,
        })
    }
}
