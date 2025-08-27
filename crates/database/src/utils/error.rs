use actix_failwrap::ErrorResponse;
use sqlx::Error as SqlxError;
use thiserror::Error as ThisError;

use super::connection::DatabaseConnectionError;

pub type ModelResult<T> = Result<T, DatabaseError>;

#[derive(ThisError, Debug, ErrorResponse)]
pub enum DatabaseError {
    #[error("{0:#}")]
    DatabaseQuery(#[from] SqlxError),

    #[error("{0:#}")]
    DatabaseConnectionError(#[from] DatabaseConnectionError),

    #[status_code(NotFound)]
    #[error("No {0} found with that query.")]
    ModelNotFound(&'static str),

    #[status_code(BadRequest)]
    #[error("Failed to parse UUID: {0}")]
    UuidParseError(#[from] uuid::Error),

    #[status_code(BadRequest)]
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
}
