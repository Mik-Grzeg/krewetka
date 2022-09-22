use std::path::Path;

use lib::migrator::clickhouse::ClickhouseMigrations;
use lib::migrator::migrate::AbstractMigratorSql;
use lib::storage::clickhouse::ClickhouseState;

use log::info;
use lazy_static::lazy_static;

lazy_static!{
    static ref MIGRATIONS_DIR: &'static Path = &Path::new("./migrations");
}

#[tokio::main]
async fn main() {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    info!("Starting migrator");

    let ch_state = ClickhouseState::new("clickhouse".to_string(), 9000, "myuser".to_string(), "password".to_string());

    let mut migrator = ClickhouseMigrations::new(ch_state);
    if let None = migrator.get_migrations_from_dir(&MIGRATIONS_DIR) { panic!("unable to get migartion files")}

    migrator.init_migration_info_persistant().await;
}
