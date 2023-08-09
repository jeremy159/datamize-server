use axum::{extract::State, Json};
use ynab::types::Account;

use crate::error::HttpJsonDatamizeResult;
use crate::services::budget_providers::{YnabAccountService, YnabAccountServiceExt};

/// Returns all accounts from YNAB's API.
#[tracing::instrument(name = "Get all accounts from YNAB's API", skip_all)]
pub async fn get_ynab_accounts(
    State(mut ynab_account_service): State<YnabAccountService>,
) -> HttpJsonDatamizeResult<Vec<Account>> {
    Ok(Json(ynab_account_service.get_all_ynab_accounts().await?))
}
