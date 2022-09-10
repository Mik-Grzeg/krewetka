


use crate::importers::import::Import;



use self::config::{ConfigCache, ConfigErr};
use log::{debug, error, info};

use tokio::sync::mpsc;
use tokio::task::{self};


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

    async fn init_components(config: Configuration) -> Result<(), AppInitErr> {
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
        let importer_task = task::spawn(async move {
            info!("Spawned importer...");

            let tx = tx.clone();
            loop {
                match importer.import().await {
                    Ok(m) => {
                        if let Err(e) = tx.send(m).await {
                            error!(
                                "unable to send fetched message to an exporter channel: {:?}",
                                e
                            );
                            break;
                        };
                    }
                    Err(e) => {
                        error!("unable to fetch message: {:?}", e);
                        continue;
                    }
                };
            }
            tx.closed();
        });

        // spawning task responsible for exporting data
        let exporter_task = task::spawn(async move {
            info!("Spawned exporter...");
            exporters::run(exporter, &mut rx).await
        });

        // TODO remove that stuff
        let first = importer_task.await.unwrap();
        let second = exporter_task.await.unwrap();
        debug!("First: {:?}\tSecond: {:?}", first, second);

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

#[tokio::main]
async fn main() {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    // parse configs
    let (config_cache, config) = init_config().expect("Configuration init failed");
    let app_state = ApplicationState::new(config_cache, config)
        .expect("Unable to initialize application state");

    let _ = ApplicationState::init_components(app_state.config().unwrap()).await;
}
