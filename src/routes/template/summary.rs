use axum::http::StatusCode;

/// Returns a budget template summary.
pub async fn template_summary() -> StatusCode {
    StatusCode::OK
}
