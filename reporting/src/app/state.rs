use super::config::{AppConfigCache, Settings};
use super::errors::ConfigErr;

// use super::models::DbLayer;
// use super::models::DBAccessor;

#[derive(Debug, Clone)]
pub struct State {
    pub cfg_cache: AppConfigCache,
}

impl State {
    pub fn config(&self) -> Result<Settings, ConfigErr> {
        self.cfg_cache.get_config::<Settings>()
    }

    pub async fn new() -> Result<Self, ConfigErr> {
        let config_cache = AppConfigCache::new()?;
        // let cfg = config_cache.get_config::<Settings>()?;

        // let db_pool = Arc::new(init_db(
        //     &cfg.clickhouse_host,
        //     cfg.clickhouse_port,
        //     &cfg.clickhouse_user,
        //     &cfg.clickhouse_password,
        // ));

        Ok(Self {
            cfg_cache: config_cache,
        })
    }
}
