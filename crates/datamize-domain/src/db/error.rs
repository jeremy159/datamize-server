/// Database errors.  Any unexpected errors that come from the database are classified as
/// `BackendError`, but errors we know about have more specific types.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DbError {
    /// Indicates that a request to create an entry failed because it already exists.
    #[error("Already exists")]
    AlreadyExists,

    /// Catch-all error type for unexpected database errors.
    #[error("Database error: {0}")]
    BackendError(String),

    /// Indicates a failure processing the data that already exists in the database.
    #[error("Data integrity error: {0}")]
    DataIntegrityError(String),

    /// Indicates that a requested entry does not exist.
    #[error("Entity not found")]
    NotFound,

    /// Indicates that the database is not available (maybe because of too many active concurrent
    /// connections).
    #[error("Unavailable")]
    Unavailable,
}

/// Result type for this module.
pub type DbResult<T> = Result<T, DbError>;

impl From<sqlx::Error> for DbError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::ColumnDecode { source, .. } => {
                Self::DataIntegrityError(source.to_string())
            }
            sqlx::Error::RowNotFound => Self::NotFound,
            e if e.to_string().contains("FOREIGN KEY constraint failed") => Self::NotFound,
            e if e.to_string().contains("UNIQUE constraint failed") => Self::AlreadyExists,
            e => Self::BackendError(e.to_string()),
        }
    }
}

impl From<fred::error::RedisError> for DbError {
    fn from(e: fred::error::RedisError) -> Self {
        match e {
            e if *e.kind() == fred::error::RedisErrorKind::NotFound => Self::NotFound,
            e if e.details().starts_with("WRONGTYPE") => {
                Self::DataIntegrityError(e.details().to_string())
            }
            _ => Self::BackendError(e.to_string()),
        }
    }
}

impl From<uuid::Error> for DbError {
    fn from(value: uuid::Error) -> Self {
        Self::DataIntegrityError(value.to_string())
    }
}

impl From<serde_json::Error> for DbError {
    fn from(value: serde_json::Error) -> Self {
        Self::DataIntegrityError(value.to_string())
    }
}
