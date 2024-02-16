use axum::extract::State;
use datamize_domain::ExternalAccount;

use crate::error::{AppJson, HttpJsonDatamizeResult};
use crate::services::budget_providers::DynExternalAccountService;

/// Returns all external accounts. Those are accounts that can be web scrapped.
#[tracing::instrument(name = "Get all external accounts", skip_all)]
pub async fn get_external_accounts(
    State(external_account_service): State<DynExternalAccountService>,
) -> HttpJsonDatamizeResult<Vec<ExternalAccount>> {
    Ok(AppJson(
        external_account_service.get_all_external_accounts().await?,
    ))
}
