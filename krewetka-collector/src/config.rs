use config::builder::DefaultState;
use config::{Config, ConfigBuilder, Environment, File};
use serde::Deserialize;
use std::time::SystemTime;

const DEFAULT_ENV_VAR_PREFIX: &str = "KREWETKA";

#[derive(Debug)]
pub enum ConfigErr {
    Read(config::ConfigError),
}

pub struct ConfigCache {
    config: Config,
    config_path: String,
    ts: SystemTime,
}

impl ConfigCache {
    pub fn new(global_config_path: &str) -> Result<Self, ConfigErr> {
        let config_cache = Self {
            config: Self::load_config(global_config_path)?,
            config_path: global_config_path.to_owned(),
            ts: SystemTime::now(),
        };

        Ok(config_cache)
    }

    fn load_config(global_config_path: &str) -> Result<Config, ConfigErr> {
        let base_config_builder = ConfigBuilder::<DefaultState>::default();
        base_config_builder
            .add_source(File::with_name(global_config_path).required(false))
            .add_source(Environment::with_prefix(DEFAULT_ENV_VAR_PREFIX).separator("__"))
            .build()
            .map_err(ConfigErr::Read)
    }

    pub fn get_config<'d, T: Deserialize<'d>>(&self) -> Result<T, ConfigErr> {
        self.config
            .clone()
            .try_deserialize()
            .map_err(ConfigErr::Read)
    }
}
