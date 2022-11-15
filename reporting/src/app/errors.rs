use clickhouse_rs::errors::Error;
use std::io::Error as IoErr;

#[derive(Debug)]
pub enum ConfigErr {
    Read(config::ConfigError),
    MissingNeccessarySetting(String),
}

impl From<ConfigErr> for IoErr {
    fn from(c: ConfigErr) -> Self {
        match c {
            ConfigErr::Read(e) => {
                IoErr::new::<config::ConfigError>(std::io::ErrorKind::Unsupported, e)
            }
            ConfigErr::MissingNeccessarySetting(e) => {
                IoErr::new::<String>(std::io::ErrorKind::InvalidInput, e)
            }
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    DBAccessorError(String),
    DbCouldNotGetHandler(clickhouse_rs::errors::Error),
}

impl From<Error> for AppError {
    fn from(e: Error) -> Self {
        Self::DbCouldNotGetHandler(e)
    }
}
