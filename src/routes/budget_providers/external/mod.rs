use axum::{routing::get, Router};

use crate::startup::AppState;

mod accounts;

use accounts::*;

pub fn get_external_routes() -> Router<AppState> {
    Router::new().route("/accounts", get(get_external_accounts))
}
