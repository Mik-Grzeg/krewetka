use crate::importers::Import;

use self::config::{ConfigCache, ConfigErr};
use log::{debug, error, info};

use tokio::sync::mpsc;
use tokio::task::{self};
use std::sync::Arc;

mod config;
mod exporters;
mod importers;
mod settings;
pub use importers::ZMQ;

use settings::Configuration;

const CONFIG_PATH: &str = "./krewetka.yaml";

pub struct ApplicationState {
    pub config: ConfigCache,
}

#[derive(Debug)]
pub enum AppInitErr {
    Config(ConfigErr),
    ImporterInit(ConfigErr),
}

impl ApplicationState {
    pub fn new(config_cache: ConfigCache, _config: Configuration) -> Result<Self, AppInitErr> {
        Ok(Self {
            config: config_cache,
        })
    }

    pub fn config(&self) -> Result<Configuration, ConfigErr> {
        self.config.get_config::<Configuration>()
    }

    pub async fn init_components(config: Configuration) -> Result<(), AppInitErr> {
        // initialize exporter and importer
        let exporter = config
            .exporter
            .destination
            .construct_exporter(config.exporter.settings)
            .expect("unable to initialize exporter");

        let importer = config
            .importer
            .source
            .construct_importer(config.importer.settings)
            .expect("unable to initialize importer");

        // make a shared channel for common data
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(20);


        // spawning task responsbile for importing data
        let importer_task = task::spawn( async move {
             importers::run(importer, tx).await
        });
        

        // spawning task responsible for exporting data
        let exporter_task = task::spawn( async move {
            exporters::run(exporter, &mut rx).await
        });

        Ok(())
    }
}

pub fn init_config() -> Result<(ConfigCache, Configuration), AppInitErr> {
    let config_cache = ConfigCache::new(&CONFIG_PATH).map_err(AppInitErr::Config)?;
    let configuration = config_cache
        .get_config::<Configuration>()
        .map_err(AppInitErr::Config)?;

    Ok((config_cache, configuration))
}