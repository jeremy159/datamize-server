use axum::extract::State;
use ynab::types::Account;

use crate::{
    error::{AppJson, HttpJsonDatamizeResult},
    services::budget_providers::DynYnabAccountService,
};

/// Returns all accounts from YNAB's API.
#[tracing::instrument(name = "Get all accounts from YNAB's API", skip_all)]
pub async fn get_ynab_accounts(
    State(ynab_account_service): State<DynYnabAccountService>,
) -> HttpJsonDatamizeResult<Vec<Account>> {
    Ok(AppJson(ynab_account_service.get_all_ynab_accounts().await?))
}
