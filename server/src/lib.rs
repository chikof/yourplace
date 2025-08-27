use std::io::Error as IoError;

use actix_failwrap::ErrorResponse;
use flexi_logger::FlexiLoggerError;
use thiserror::Error as ThisError;

pub mod routes;

#[derive(Debug, ThisError, ErrorResponse)]
pub enum AppError {
    #[error("{0:#}")]
    Io(#[from] IoError),

    #[error("0:#")]
    LoggerError(#[from] FlexiLoggerError),

    #[status_code(Unauthorized)]
    #[error("You are not authorized to access this resource")]
    AuthorizationError,
}
