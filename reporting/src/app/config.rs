use super::errors::ConfigErr;
use config::builder::DefaultState;
use config::{Config, ConfigBuilder, Environment};
use serde::Deserialize;

const DEFAULT_ENV_VAR_PREFIX: &str = "KREWETKA";

#[derive(Debug, Clone)]
pub struct AppConfigCache {
    config: Config,
}

impl AppConfigCache {
    pub fn new() -> Result<Self, ConfigErr> {
        let config_cache = Self {
            config: Self::load_config()?,
        };

        Ok(config_cache)
    }

    pub fn get_config<'d, T: Deserialize<'d>>(&self) -> Result<T, ConfigErr> {
        self.config
            .clone()
            .try_deserialize()
            .map_err(ConfigErr::Read)
    }

    fn load_config() -> Result<Config, ConfigErr> {
        let base_config_builder = ConfigBuilder::<DefaultState>::default();
        base_config_builder
            .add_source(Environment::with_prefix(DEFAULT_ENV_VAR_PREFIX).separator("__"))
            .build()
            .map_err(ConfigErr::Read)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub clickhouse_user: String,
    pub clickhouse_password: String,
    pub clickhouse_host: String,
    pub clickhouse_port: u16,
}
