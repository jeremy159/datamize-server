use axum::extract::State;
use ynab::Payee;

use crate::{
    error::{AppJson, HttpJsonDatamizeResult},
    services::budget_providers::DynYnabPayeeService,
};

/// Returns all accounts from YNAB's API.
#[tracing::instrument(name = "Get all payees from YNAB's API", skip_all)]
pub async fn get_ynab_payees(
    State(ynab_payee_service): State<DynYnabPayeeService>,
) -> HttpJsonDatamizeResult<Vec<Payee>> {
    Ok(AppJson(ynab_payee_service.get_all_ynab_payees().await?))
}
