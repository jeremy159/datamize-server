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

impl From<redis::RedisError> for DbError {
    fn from(e: redis::RedisError) -> Self {
        match e.kind() {
            redis::ErrorKind::AuthenticationFailed
            | redis::ErrorKind::BusyLoadingError
            | redis::ErrorKind::ClusterDown
            | redis::ErrorKind::MasterDown => Self::Unavailable,
            redis::ErrorKind::ResponseError | redis::ErrorKind::TypeError => {
                Self::DataIntegrityError(e.to_string())
            }
            _ => Self::BackendError(e.to_string()),
        }
    }
}
