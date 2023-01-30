use axum::http::StatusCode;

/// Returns a budget template details
pub async fn template_details() -> StatusCode {
    StatusCode::OK
}
