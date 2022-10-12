use crate::actors::storage::clickhouse::ClickhouseState;
use crate::migrator::creator::hasher;
use crate::migrator::migrate::{AbstractMigratorSql, MigrationFiles, MigratorError};
use async_trait::async_trait;
use clickhouse_rs::{Block, ClientHandle};
use log::{error, info, warn};
use std::fs;
use std::fs::{DirEntry, File};
use std::io::Read;
use std::path::{Path, PathBuf};

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

    async fn apply_migration(
        &self,
        path: &PathBuf,
        client: &mut ClientHandle,
    ) -> Result<(), MigratorError> {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        info!("reading migration script: {}", file_name);

        let ddl = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                error!("unable to read: {}", file_name);
                return Err(MigratorError::ReadingMigrationFilesFailed(Box::new(e)));
            }
        };

        client
            .execute(&ddl)
            .await
            .map_err(|e| MigratorError::ApplyFailed(Box::new(e)))?;

        self.mark_migration_as_applied(&ddl, path, client).await?;
        Ok(())
    }

    async fn mark_migration_as_applied(
        &self,
        ddl: &str,
        path: &PathBuf,
        client: &mut ClientHandle,
    ) -> Result<(), MigratorError> {
        let path = path.to_str().unwrap();
        let block = Block::new()
            .column("migration_hash", vec![hasher(ddl)])
            .column("migration_file_name", vec![path]);

        client
            .insert("_db_migrations", block)
            .await
            .map_err(|e| MigratorError::SettingsMigrationAsAppliedFailed(Box::new(e)))?;

        info!("{} migration set as applied", path);

        Ok(())
    }

    async fn ignore_applied_migrations(
        &self,
        client: &mut ClientHandle,
        migration_files: &mut Vec<PathBuf>,
    ) {
        let dql = r#"
            SELECT * FROM _db_migrations;
        "#;
        let block = client
            .query(dql)
            .fetch_all()
            .await
            .expect("Unable to fetch applied migrations from table _db_migrations")
            .rows()
            .map(|r| -> String {
                r.get("migration_file_name")
                    .expect("missing migration_file_name in result")
            })
            .collect::<Vec<String>>();

        migration_files.retain(|f| {
            let f_name = f.to_str().unwrap();
            let applied = block.iter().any(|i| i == f_name);
            if applied {
                info!("Migration from file {} already applied", f_name)
            };
            !applied
        });
    }
}

#[async_trait]
impl AbstractMigratorSql for ClickhouseMigrations {
    async fn run_migrations(&self) -> Result<(), MigratorError> {
        let mut client = self
            .clickhouse_state
            .pool
            .as_ref()
            .get_handle()
            .await
            .map_err(|e| MigratorError::InitializationFailed(Box::new(e)))?;

        let mut migrations = self
            .validated_migration_scripts
            .as_ref()
            .unwrap()
            .recognized_files
            .clone();

        let inital_number_of_migration_files = migrations.len();
        info!("{} migration files found", inital_number_of_migration_files);
        self.ignore_applied_migrations(&mut client, &mut migrations)
            .await;
        info!(
            "{} migration has already been applied",
            inital_number_of_migration_files - migrations.len()
        );

        for path in migrations {
            self.apply_migration(&path, &mut client).await?;
        }

        Ok(())
    }

    fn get_migrations_from_dir(&mut self, dir: &Path) -> Option<()> {
        let mut files_sorted_by_timestamp = dir
            .read_dir()
            .expect("reading directory failed")
            .filter_map(|f| match f {
                Ok(f) => Some(f),
                Err(_e) => None,
            })
            .collect::<Vec<DirEntry>>();

        files_sorted_by_timestamp
            .sort_by(|f1, f2| f1.file_name().partial_cmp(&f2.file_name()).unwrap());

        // validate files
        let mut migration_scripts = MigrationFiles {
            recognized_files: Vec::new(),
        };
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

        info!(
            "Matched and validated {} files",
            migration_scripts.recognized_files.len()
        );
        if !migration_scripts.recognized_files.is_empty() {
            self.validated_migration_scripts = Some(migration_scripts);
            return Some(());
        }
        None
    }

    fn prepare_migrations(&self) {
        let mut migration_to_apply = String::new();
        for script in &self
            .validated_migration_scripts
            .as_ref()
            .unwrap()
            .recognized_files
        {
            let mut f = match File::open(script) {
                Ok(f) => f,
                Err(_e) => {
                    error!("Unable to open file: {}", script.display());
                    continue;
                }
            };

            if let Err(_e) = f.read_to_string(&mut migration_to_apply) {
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
            Err(_e) => panic!("create migration table if not exists [FAILED]"),
        };

        Ok(())
    }
}
