use axum::Router;

use crate::startup::AppState;

mod balance_sheet;
mod budget_providers;
mod budget_template;

use balance_sheet::*;
use budget_providers::*;
use budget_template::*;

pub fn get_api_routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .nest("/template", get_budget_template_routes(app_state))
        .nest("/balance_sheet", get_balance_sheets_routes(app_state))
        .nest("/budget_providers", get_budget_providers_routes(app_state))
}
