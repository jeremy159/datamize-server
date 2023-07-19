mod budgeter;
mod budgeters;
mod common;
mod details;
mod expense_categorization;
mod expenses_categorization;
mod external_expense;
mod external_expenses;
mod summary;
mod transactions;

use axum::{
    routing::{get, post},
    Router,
};
use budgeter::*;
use budgeters::*;
use details::*;
use expense_categorization::*;
use expenses_categorization::*;
use external_expense::*;
use external_expenses::*;
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
        .route("/external_expenses", get(get_all_external_expenses))
        .route("/external_expense", post(create_external_expense))
        .route(
            "/external_expense/:external_expense_id",
            get(get_external_expense)
                .put(update_external_expense)
                .delete(delete_external_expense),
        )
        .route(
            "/expenses_categorization",
            get(get_all_expenses_categorization).put(update_all_expenses_categorization),
        )
        .route(
            "/expense_categorization/:expense_categorization_id",
            get(get_expense_categorization).put(update_expense_categorization),
        )
}
