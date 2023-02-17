use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub type HttpJsonAppResult<T> = Result<Json<T>, AppError>;

#[derive(Debug)]
pub enum AppError {
    InternalServerError(anyhow::Error),
    ResourceNotFound,
    YearAlreadyExist,
    MonthAlreadyExist,
    ValidationError,
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::InternalServerError(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError(inner) => {
                tracing::error!("AppError::InternalServerError: {}", inner.to_string());
                tracing::debug!("stacktrace: {}", inner.backtrace());
                (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
            }
            AppError::ValidationError => (StatusCode::UNPROCESSABLE_ENTITY, "validation errors"),
            AppError::ResourceNotFound => (StatusCode::NOT_FOUND, "resource does not exist"),
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
