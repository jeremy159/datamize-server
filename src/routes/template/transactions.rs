use axum::http::StatusCode;

/// Returns a budget template transactions, i.e. all the scheduled transactions in the upcoming month.
pub async fn template_transactions() -> StatusCode {
    StatusCode::OK
}
