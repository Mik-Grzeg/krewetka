use std::path::{Path, PathBuf};
use std::ffi::OsString;
use regex::Regex;
use lazy_static::lazy_static;
use async_trait::async_trait;
use std::error::Error;

pub enum MigratorError {
    InitializationFailed(Box<dyn Error>),
}

pub struct MigrationFiles {
    pub recognized_files: Vec<PathBuf>,
}


#[async_trait]
pub trait AbstractMigratorSql {
    async fn init_migration_info_persistant(&self) -> Result<(), MigratorError>;
    fn get_migrations_from_dir(&mut self, dir: &Path) -> Option<()>;
    fn prepare_migrations(&self) -> ();
    fn validate(file_name: &OsString) -> bool {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\[0-9]{10}\..*").unwrap();
        }

        RE.is_match(file_name.to_str().unwrap())
    }
}
