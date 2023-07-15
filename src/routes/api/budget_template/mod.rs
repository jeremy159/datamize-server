mod budgeter;
mod budgeters;
mod common;
mod details;
mod summary;
mod transactions;

use axum::{
    routing::{get, post},
    Router,
};
use budgeter::*;
use budgeters::*;
use details::*;
use summary::*;
use transactions::*;

use crate::startup::AppState;

pub fn get_budget_template_routes() -> Router<AppState> {
    Router::new()
        .route("/details", get(template_details))
        .route("/summary", get(template_summary))
        .route("/transactions", get(template_transactions))
        .route("/budgeters", get(get_all_budgeters))
        .route("/budgeter", post(create_budgeter))
        .route(
            "/budgeter/:budgeter_id",
            get(get_budgeter)
                .put(update_budgeter)
                .delete(delete_budgeter),
        )
}
