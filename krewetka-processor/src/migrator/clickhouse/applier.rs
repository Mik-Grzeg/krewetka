use crate::migrator::migrate::{AbstractMigratorSql, MigrationFiles, MigratorError};
use crate::storage::clickhouse::ClickhouseState;
use std::io::Read;
use std::path::Path;
use std::fs::{File, ReadDir, DirEntry};
use async_trait::async_trait;
use log::{info, warn, debug, error};


pub struct ClickhouseMigrations {
    validated_migration_scripts: Option<MigrationFiles>,
    clickhouse_state: ClickhouseState,
}

impl ClickhouseMigrations {
    pub fn new(state: ClickhouseState) -> Self {
        Self {
            validated_migration_scripts: None,
            clickhouse_state: state,
        }
    }
}

#[async_trait]
impl AbstractMigratorSql for ClickhouseMigrations {
    fn get_migrations_from_dir(&mut self, dir: &Path) -> Option<()>{

        let mut files_sorted_by_timestamp = dir
            .read_dir()
            .expect("reading directory failed")
            .filter_map(|f|  match f {
                Ok(f) => Some(f),
                Err(e) => None,
            })
            .collect::<Vec<DirEntry>>();

            files_sorted_by_timestamp
            .sort_by(|f1, f2| f1.file_name().partial_cmp(&f2.file_name()).unwrap());


        // validate files
        let mut migration_scripts = MigrationFiles{ recognized_files: Vec::new() };
        for file in files_sorted_by_timestamp {
            let file_name = file.file_name();

            if ClickhouseMigrations::validate(&file_name) {
                info!("[{}] path validation PASSED", file_name.to_str().unwrap());
                let f = file.path();
                migration_scripts.recognized_files.push(f);
            } else {
                warn!("[{}] path validation FAILED", file_name.to_str().unwrap());
            }
        }

        if migration_scripts.recognized_files.is_empty() {
            self.validated_migration_scripts = Some(migration_scripts);
            return Some(())
        }
        None
    }

    fn prepare_migrations(&self) -> () {
        let mut migration_to_apply = String::new();
        for script in &self.validated_migration_scripts.as_ref().unwrap().recognized_files {
            let mut f = match File::open(script) {
                Ok(f) => f,
                Err(e) =>  { error!("Unable to open file: {}", script.display()); continue},
            };

            if let Err(e) = f.read_to_string(&mut migration_to_apply) {
                error!("Unable to read file: {}", script.display());
            }
        }
    }

    async fn init_migration_info_persistant(&self) -> Result<(), MigratorError> {
        let mut client = self
            .clickhouse_state
            .pool
            .as_ref()
            .get_handle()
            .await
            .map_err(|e| MigratorError::InitializationFailed(Box::new(e)))?;

        let ddl = r#"
            CREATE TABLE IF NOT EXISTS _db_migrations (
                `migration_hash` UInt64,
                `migration_file_name` String,
                `insert_time` DateTime DEFAULT now()
            )
            ENGINE = MergeTree()
            ORDER BY (insert_time)
        "#;
        match client.execute(ddl).await {
            Ok(_) => info!("create migration table if not exists [PASSED]"),
            Err(e) =>  panic!("create migration table if not exists [FAILED]"),
        };

        Ok(())
    }
}
