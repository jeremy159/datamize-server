mod accounts;

use accounts::*;
use axum::{routing::get, Router};

use crate::startup::AppState;

pub fn get_ynab_routes() -> Router<AppState> {
    Router::new().route("/accounts", get(get_ynab_accounts))
}
