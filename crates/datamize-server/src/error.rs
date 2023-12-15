use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use datamize_domain::db::DbError;
use serde_json::json;

pub type DatamizeResult<T> = Result<T, AppError>;
pub type HttpJsonDatamizeResult<T> = Result<Json<T>, AppError>;

#[derive(thiserror::Error)]
pub enum AppError {
    #[error("resource does not exist")]
    ResourceNotFound,
    #[error("resource already exist")]
    ResourceAlreadyExist,
    #[error("Data is corrupted or invalid")]
    DataIntegrityError(String),
    #[error("validation errors")]
    ValidationError,
    #[error("Error in the YNAB API")]
    YnabError(#[from] ynab::Error),
    #[error(transparent)]
    InternalServerError(#[from] anyhow::Error),
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<DbError> for AppError {
    fn from(value: DbError) -> Self {
        match value {
            DbError::NotFound => AppError::ResourceNotFound,
            DbError::AlreadyExists => AppError::ResourceAlreadyExist,
            DbError::DataIntegrityError(e) => AppError::DataIntegrityError(e),
            e => AppError::InternalServerError(e.into()),
        }
    }
}

impl From<config::ConfigError> for AppError {
    fn from(value: config::ConfigError) -> Self {
        AppError::InternalServerError(value.into())
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(value: chrono::ParseError) -> Self {
        AppError::InternalServerError(value.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError(ref inner) => {
                tracing::error!("AppError::InternalServerError: {:?}", self);
                tracing::debug!("stacktrace: {}", inner.backtrace());
                (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
            }
            AppError::DataIntegrityError(e) => {
                tracing::error!("AppError::DataIntegrityError: {:?}", e);
                (StatusCode::BAD_REQUEST, "Data is corrupted or invalid")
            }
            AppError::ValidationError => (StatusCode::UNPROCESSABLE_ENTITY, "validation errors"),
            AppError::ResourceNotFound => (StatusCode::NOT_FOUND, "resource does not exist"),
            AppError::ResourceAlreadyExist => (StatusCode::CONFLICT, "resource already exist"),
            AppError::YnabError(ref e) => {
                tracing::error!("AppError::YnabError: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong with ynab api",
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[derive(Debug)]
pub enum JsonError {
    JsonExtractorRejection(JsonRejection),
}

impl From<JsonRejection> for JsonError {
    fn from(value: JsonRejection) -> Self {
        Self::JsonExtractorRejection(value)
    }
}

impl IntoResponse for JsonError {
    fn into_response(self) -> axum::response::Response {
        let payload = json!({
            "message": format!("{:?}", self),
            "origin": "with_rejection"
        });
        let code = match self {
            JsonError::JsonExtractorRejection(x) => match x {
                JsonRejection::JsonDataError(_) => StatusCode::UNPROCESSABLE_ENTITY,
                JsonRejection::JsonSyntaxError(_) => StatusCode::BAD_REQUEST,
                JsonRejection::MissingJsonContentType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
        };
        (code, Json(payload)).into_response()
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
