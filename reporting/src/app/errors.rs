use actix_web::http::{header::ContentType, StatusCode};
use actix_web::HttpResponse;
use clickhouse_rs::errors::Error as ChErr;
use derive_more::{Display, Error};
use log::error;
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
    JsonSerialization(String),
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
