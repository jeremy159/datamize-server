use axum::{
    extract::{rejection::JsonRejection, FromRequest},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use config::ConfigError;
use datamize_domain::db::DbError;
use serde::Serialize;

pub type DatamizeResult<T> = Result<T, AppError>;
pub type HttpJsonDatamizeResult<T> = Result<AppJson<T>, AppError>;

#[derive(FromRequest, Debug)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

#[derive(thiserror::Error)]
pub enum AppError {
    #[error("The request body contained invalid JSON")]
    JsonRejection(#[from] JsonRejection),
    #[error("Resource does not exist")]
    ResourceNotFound,
    #[error("Resource already exist")]
    ResourceAlreadyExist,
    #[error("Error with Database interaction")]
    DbError(#[from] DbError),
    #[error("Error with the Configuration")]
    ConfigError(#[from] ConfigError),
    #[error("Error with Date conversion")]
    ParseError(#[from] chrono::ParseError),
    #[error("Error in the YNAB API")]
    YnabError(#[from] ynab::Error),
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // How we want errors responses to be serialized
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message) = match self {
            AppError::JsonRejection(rejection) => {
                // This error is caused by bad user input so don't log it
                (rejection.status(), rejection.body_text())
            }
            AppError::ResourceNotFound => {
                (StatusCode::NOT_FOUND, "Resource does not exist".to_owned())
            }
            AppError::ResourceAlreadyExist => {
                (StatusCode::CONFLICT, "Resource already exist".to_owned())
            }
            AppError::DbError(err) => {
                tracing::error!(?err, "error from database");
                match err {
                    DbError::NotFound => {
                        (StatusCode::NOT_FOUND, "Resource does not exist".to_owned())
                    }
                    DbError::AlreadyExists => {
                        (StatusCode::CONFLICT, "Resource already exist".to_owned())
                    }
                    DbError::DataIntegrityError(_) => (
                        StatusCode::BAD_REQUEST,
                        "Data is corrupted or invalid".to_owned(),
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Something went wrong".to_owned(),
                    ),
                }
            }
            AppError::ConfigError(err) => {
                tracing::error!(?err, "error from config");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
            AppError::ParseError(err) => {
                tracing::error!(?err, "error from date parsing");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
            AppError::YnabError(err) => {
                tracing::error!(?err, "error from ynab api");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
        };

        (status, AppJson(ErrorResponse { message })).into_response()
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
