use std::error;

use clap::Parser;
use ::config::Config;
use crate::importers::import::Import;

use self::config::{ConfigCache, ConfigErr};
use log::{info, debug, warn, error};

use tokio::task::{self, futures};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

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


        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(5);


        let (first, second) = tokio::join!(task::spawn( async move {
            debug!("Mega tsk spawned");
            let mut arr = vec![];
            for n in 1..30 {

                arr.push(n);
                tx.send(arr.clone()).await;
                info!("Sent: {}", n);
                sleep(Duration::from_millis(100)).await;
            }

            tx.closed();
        }),
        task::spawn( async move {
            let mut buffer:Vec<Vec<u8>> = Vec::with_capacity(10);

            loop {
                let msg = match rx.recv().await {
                    Some(m) => m,
                    None =>  { error!("We've been tricked and quite possibly bamboozled. No message was found on the channel"); return  }
                };

                buffer.push(msg);
                info!("Updated buffer: {:?}", buffer);
                sleep(Duration::from_millis(1000)).await;
                debug!("Len: {}\tCapacity: {}", buffer.len(), buffer.capacity());
                if buffer.len() == buffer.capacity()-1 { buffer.clear() };

            }

        }));


        debug!("First: {:?}\tSecond: {:?}", first, second);

        Ok(())
        // let message = match importer.import() {
        //     Ok(m) => m,
        //     Err(e) => {
        //         error!("unable receive message");
        //         continue;
        //     }
        // };
        // debug!("Received message: {}", String::from_utf8_lossy(&message));

        // match exporter.export(message).await {
        //     Ok((a, b)) => debug!("partition: {}\toffset: {}", a, b),
        //     Err(e) => error!("exporter err: {:?}", e)
        // };
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
