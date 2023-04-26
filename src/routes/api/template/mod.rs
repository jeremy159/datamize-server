mod common;
mod details;
mod summary;
mod transactions;

use axum::{routing::get, Router};
use details::*;
use summary::*;
use transactions::*;

use crate::startup::AppState;

pub fn get_budget_template_routes() -> Router<AppState> {
    Router::new()
        .route("/details", get(template_details))
        .route("/summary", get(template_summary))
        .route("/transactions", get(template_transactions))
}
