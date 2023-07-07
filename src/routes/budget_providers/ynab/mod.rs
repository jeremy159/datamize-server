mod accounts;
mod payees;

use accounts::*;
use axum::{routing::get, Router};
use payees::*;

use crate::startup::AppState;

pub fn get_ynab_routes() -> Router<AppState> {
    Router::new()
        .route("/accounts", get(get_ynab_accounts))
        .route("/payees", get(get_ynab_payees))
}
