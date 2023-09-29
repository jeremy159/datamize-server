use axum::{extract::State, Json};

use crate::error::HttpJsonDatamizeResult;
use crate::models::budget_providers::ExternalAccount;
use crate::services::budget_providers::DynExternalAccountService;

/// Returns all external accounts. Those are accounts that can be web scrapped.
#[tracing::instrument(name = "Get all external accounts", skip_all)]
pub async fn get_external_accounts(
    State(external_account_service): State<DynExternalAccountService>,
) -> HttpJsonDatamizeResult<Vec<ExternalAccount>> {
    Ok(Json(
        external_account_service.get_all_external_accounts().await?,
    ))
}
