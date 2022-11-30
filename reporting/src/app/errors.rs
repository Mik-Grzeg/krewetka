use actix_web::http::{header::ContentType, StatusCode};
use actix_web::HttpResponse;
use clickhouse_rs::errors::Error as ChErr;
use derive_more::{Display, Error};
use log::error;
use std::io::Error as IoErr;

/// Error that may occurs before starting the actual application
/// It is generated while trying to configure all the modules
#[derive(Debug)]
pub enum ConfigErr {
    /// Reading [config::Config] fails
    Read(config::ConfigError),

    /// Some of the necessary settings are missing after parsing config
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

/// AppError is an Error that may occur while the application is working
#[derive(Debug)]
pub enum AppError {
    /// Quering database failed
    DBAccessorError(String),

    /// Acquiring database connector from pool failed
    DbCouldNotGetHandler(clickhouse_rs::errors::Error),

    /// Json serialization failed
    JsonSerialization(String),

    /// Not implemented error
    Unimplemented,
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        error!(target: "serialization_to_json", "{e}");
        Self::JsonSerialization(e.to_string())
    }
}

impl From<ChErr> for AppError {
    fn from(e: ChErr) -> Self {
        error!(target: "clickhouse_errors", "{e}");
        Self::DbCouldNotGetHandler(e)
    }
}

/// Capability to convert [AppError] to [ResponderErr]
impl From<AppError> for ResponderErr {
    fn from(e: AppError) -> Self {
        match e {
            AppError::DBAccessorError(_)
            | AppError::DbCouldNotGetHandler(_)
            | AppError::JsonSerialization(_)
            | AppError::Unimplemented => Self::InternalError,
        }
    }
}

/// Enum which can be returned whenever the return type is [impl Responder]
#[derive(Debug, Display, Error)]
pub enum ResponderErr {
    #[display(fmt = "An internal error occurred. Please try again later.")]
    InternalError,
}

impl actix_web::error::ResponseError for ResponderErr {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
