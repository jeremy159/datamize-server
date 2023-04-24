use axum::Router;

use crate::startup::AppState;

mod external;
mod ynab;

use self::ynab::*;
use external::*;

pub fn get_budget_providers_routes() -> Router<AppState> {
    Router::new()
        .nest("/ynab", get_ynab_routes())
        .nest("/external", get_external_routes())
}
