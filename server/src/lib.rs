use std::env::VarError;
use std::io::Error as IoError;

use actix_failwrap::ErrorResponse;
use flexi_logger::FlexiLoggerError;
use oauth2::RequestTokenError;
use oauth2::url::ParseError as UrlParseError;
use reqwest::Error as ReqwestError;
use thiserror::Error as ThisError;

use database::utils::error::DatabaseError;

pub mod config;
pub mod routes;
pub mod state;

#[derive(Debug, ThisError, ErrorResponse)]
pub enum AppError {
    #[error("{0:#}")]
    Io(#[from] IoError),

    #[error("{0:#}")]
    HttpClient(#[from] ReqwestError),

    #[error("{0:#}")]
    Oauth(#[from] RequestTokenError<ReqwestError>),

    #[error("{0:#}")]
    Database(#[from] DatabaseError),

    #[error("0:#")]
    LoggerError(#[from] FlexiLoggerError),

    #[error("{0:#}")]
    UrlParse(#[from] UrlParseError),

    #[status_code(BadRequest)]
    #[error("Invalid configuration: {0}")]
    BadConfig(#[from] VarError),

    #[status_code(Unauthorized)]
    #[error("You are not authorized to access this resource")]
    AuthorizationError,
}
