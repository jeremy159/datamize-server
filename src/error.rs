use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub type DatamizeResult<T> = Result<T, AppError>;
pub type HttpJsonDatamizeResult<T> = Result<Json<T>, AppError>;

#[derive(thiserror::Error)]
pub enum AppError {
    #[error("resource does not exist")]
    ResourceNotFound,
    #[error("resource already exist")]
    ResourceAlreadyExist,
    #[error("year already exist")]
    YearAlreadyExist,
    #[error("month already exist")]
    MonthAlreadyExist,
    #[error("validation errors")]
    ValidationError,
    #[error(transparent)]
    InternalServerError(#[from] anyhow::Error),
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError::from_sqlx(value)
    }
}

impl From<ynab::Error> for AppError {
    fn from(value: ynab::Error) -> Self {
        Self::InternalServerError(value.into())
    }
}

impl AppError {
    pub fn from_sqlx(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => AppError::ResourceNotFound,
            e => AppError::InternalServerError(e.into()),
        }
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
            AppError::ValidationError => (StatusCode::UNPROCESSABLE_ENTITY, "validation errors"),
            AppError::ResourceNotFound => (StatusCode::NOT_FOUND, "resource does not exist"),
            AppError::ResourceAlreadyExist => (StatusCode::CONFLICT, "resource already exist"),
            AppError::YearAlreadyExist => (StatusCode::CONFLICT, "year already exist"),
            AppError::MonthAlreadyExist => (StatusCode::CONFLICT, "month already exist"),
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
