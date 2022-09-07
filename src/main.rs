use clap::Parser;
use ::config::Config;
use crate::importers::import::Import;

use self::config::{ConfigCache, ConfigErr};
use log::{info, debug, warn, error};

mod exporters;
mod importers;
mod config;
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
    pub fn new (config_cache: ConfigCache, config: Configuration) -> Result<Self, AppInitErr> {
        Ok(Self { config: config_cache })
    }

    pub fn config(&self) -> Result<Configuration, ConfigErr> {
        self.config.get_config::<Configuration>()
    }

    async fn init_components(config: Configuration) -> Result<(), AppInitErr> {
        let importer = config.importer.source.construct_importer(config.importer.settings).expect("unable to initialize importer");
        let exporter = config.exporter.destination.construct_exporter(config.exporter.settings).expect("unable to initialize exporter");

        loop {
            let message = match importer.import() {
                Ok(m) => m,
                Err(e) => {
                    error!("unable receive message");
                    continue;
                }
            };
            debug!("Received message: {}", String::from_utf8_lossy(&message));

            match exporter.export(message).await {
                Ok((a, b)) => debug!("partition: {}\toffset: {}", a, b),
                Err(e) => error!("exporter err: {:?}", e)
            };
        }
    }
}

pub fn init_config() -> Result<(ConfigCache, Configuration), AppInitErr> {
    let config_cache = ConfigCache::new(&CONFIG_PATH).map_err(AppInitErr::Config)?;
    let configuration = config_cache
        .get_config::<Configuration>()
        .map_err(AppInitErr::Config)?;

    Ok((config_cache, configuration))
}

#[tokio::main]
async fn main() {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    
    let (config_cache, config) = init_config().expect("Configuration init failed");
    let app_state = ApplicationState::new(config_cache, config).expect("Unable to initialize application state");

    ApplicationState::init_components(app_state.config().unwrap()).await;

    // Parse cli flags and prepare settings
    // let args = cli::Args::parse();
    // let settings = Settings::new(&args).unwrap();

    // let importer = match ZMQImporter::new(settings.importer) {
    //     Ok(importer) => importer,
    //     Err(e) => {
    //         error!("Unable to create zmq subscriber with given settings {:?}", e);
    //         return
    //     }
    // };

    // loop {
        
    //     let msg = match importer.import() {
    //         Ok(msg) => msg,
    //         Err(e) => {
    //             error!("Unable to receive message by importer");
    //             continue;
    //         }
    //     };

    //     //TODO kafka upstream
    // }
}
