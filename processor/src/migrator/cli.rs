use clap::{Parser, Subcommand};
use log::info;
use std::path::PathBuf;

use super::creator::create_migration_blank_file;

use config::builder::DefaultState;
use config::{ConfigBuilder, Environment};

use crate::actors::storage::clickhouse::ClickhouseState;
use crate::application_state::get_config;
use crate::consts::DEFAULT_ENV_VAR_PREFIX;
use crate::migrator::clickhouse::ClickhouseMigrations;
use crate::migrator::migrate::AbstractMigratorSql;
use crate::settings::MigratorSettings;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: ActionRunner,
}

#[derive(Subcommand)]
pub enum ActionRunner {
    // Create new migration file that can be applied
    Create { migrations_dir_path: PathBuf },

    // Apply migrations
    Apply { migrations_dir_path: PathBuf },
}

impl ActionRunner {
    pub async fn run() {
        let cli = Cli::parse();

        match &cli.command {
            Self::Create {
                migrations_dir_path,
            } => {
                info!("Creating new migration file in {:?}", migrations_dir_path);
                create_migration_blank_file(migrations_dir_path.to_owned());
            }
            Self::Apply {
                migrations_dir_path,
            } => {
                info!("Applying migrations from: {:?}", migrations_dir_path);
                let base_config_builder = ConfigBuilder::<DefaultState>::default();
                let config = base_config_builder
                    .add_source(Environment::with_prefix(DEFAULT_ENV_VAR_PREFIX).separator("__"))
                    .build()
                    .unwrap();

                // deserialize env config
                let deserialized_config =
                    get_config::<MigratorSettings>(&config).expect("Getting config failed");

                info!("Starting migrator");

                let ch_state = ClickhouseState::new(deserialized_config.clickhouse_settings);

                let mut migrator = ClickhouseMigrations::new(ch_state);
                if migrator
                    .get_migrations_from_dir(migrations_dir_path)
                    .is_none()
                {
                    panic!("unable to get migartion files")
                }

                migrator
                    .init_migration_info_persistant()
                    .await
                    .expect("init migration panic");
                migrator
                    .run_migrations()
                    .await
                    .expect("running migration panic");
            }
        }
    }
}
