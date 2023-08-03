use axum::{extract::State, Json};
use ynab::Payee;

use crate::error::HttpJsonDatamizeResult;
use crate::services::budget_providers::YnabPayeeServiceExt;

/// Returns all accounts from YNAB's API.
#[tracing::instrument(name = "Get all payees from YNAB's API", skip_all)]
pub async fn get_ynab_payees<YPS: YnabPayeeServiceExt>(
    State(mut ynab_payee_service): State<YPS>,
) -> HttpJsonDatamizeResult<Vec<Payee>> {
    Ok(Json(ynab_payee_service.get_all_ynab_payees().await?))
}
