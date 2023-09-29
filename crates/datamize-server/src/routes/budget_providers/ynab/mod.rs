mod accounts;
mod payees;

use accounts::*;
use axum::{routing::get, Router};
use payees::*;

use crate::services::budget_providers::{DynYnabAccountService, DynYnabPayeeService};

pub fn get_ynab_routes<S: Clone + Sync + Send + 'static>(
    ynab_account_service: DynYnabAccountService,
    ynab_payee_service: DynYnabPayeeService,
) -> Router<S> {
    Router::new()
        .merge(get_ynab_account_routes(ynab_account_service))
        .merge(get_ynab_payee_routes(ynab_payee_service))
}

fn get_ynab_account_routes<S>(ynab_account_service: DynYnabAccountService) -> Router<S> {
    Router::new()
        .route("/accounts", get(get_ynab_accounts))
        .with_state(ynab_account_service)
}

fn get_ynab_payee_routes<S>(ynab_payee_service: DynYnabPayeeService) -> Router<S> {
    Router::new()
        .route("/payees", get(get_ynab_payees))
        .with_state(ynab_payee_service)
}
