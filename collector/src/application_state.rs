use log::error;
use std::fmt;

use tokio::sync::mpsc;
use tokio::task::{self};

use crate::config::{ConfigCache, ConfigErr};
use crate::exporters;
use crate::importers;
use crate::settings::Configuration;

const CONFIG_PATH: &str = "./krewetka.yaml";

pub struct ApplicationState {
    pub config: ConfigCache,
}

#[derive(Debug)]
pub enum AppInitErr {
    Config(ConfigErr),
    ImporterInit(ConfigErr),
}

#[derive(Debug)]
pub struct HostIdentifier {
    hostname: String,
    os_release: String,
}

impl Default for HostIdentifier {
    fn default() -> Self {
        let os_release = match sys_info::os_release() {
            Ok(r) => r,
            Err(_e) => {
                error!("Unable to acquire information about os release. Setting it to unknown");
                String::from("unknown")
            }
        };

        let hostname = match sys_info::hostname() {
            Ok(h) => h,
            Err(_e) => {
                error!("unable to acquire information about hostname. Settings it to unknown");
                String::from("unknown")
            }
        };

        HostIdentifier {
            hostname,
            os_release,
        }
    }
}

impl From<&HostIdentifier> for String {
    fn from(identifier: &HostIdentifier) -> String {
        format!("{}", identifier)
    }
}

impl fmt::Display for HostIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.hostname, self.os_release)
    }
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
        let identifier = HostIdentifier::default();

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
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1024);

        let tx1 = tx.clone();

        // spawning task responsbile for importing data
        let importer_task = task::spawn(async move { importers::run(importer, tx1).await });

        // export data
        exporters::run(exporter, &mut rx, &identifier).await;

        importer_task.await;
        Ok(())
    }
}

pub fn init_config() -> Result<(ConfigCache, Configuration), AppInitErr> {
    let config_cache = ConfigCache::new(CONFIG_PATH).map_err(AppInitErr::Config)?;
    let configuration = config_cache
        .get_config::<Configuration>()
        .map_err(AppInitErr::Config)?;

    Ok((config_cache, configuration))
}
