use axum::{response::Redirect, routing::get, Router};

use crate::startup::AppState;

mod balance_sheet;
mod budget_providers;
mod budget_template;
mod fmt;
mod utils;

use balance_sheet::*;
// use budget_providers::*;
use budget_template::*;
use fmt::*;
use utils::*;

pub fn get_ui_routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { Redirect::to("/budget/summary") }))
        .nest("/budget", get_budget_template_routes(app_state))
        .nest("/balance_sheet", get_balance_sheets_routes(app_state))
    // .nest("/budget_providers", get_budget_providers_routes(app_state))
}
