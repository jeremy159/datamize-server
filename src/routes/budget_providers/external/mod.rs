use axum::{routing::get, Router};

use crate::services::budget_providers::DynExternalAccountService;

mod accounts;

use accounts::*;

pub fn get_external_routes<S>(external_account_service: DynExternalAccountService) -> Router<S> {
    Router::new()
        .route("/accounts", get(get_external_accounts))
        .with_state(external_account_service)
}
