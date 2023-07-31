use axum::{extract::State, Json};

use crate::db::budget_providers::external::get_all_external_accounts;
use crate::error::{AppError, HttpJsonDatamizeResult};
use crate::startup::AppState;
use crate::web_scraper::account::ExternalAccount;

/// Returns all external accounts. Those are accounts that can be web scrapped.
#[tracing::instrument(name = "Get all external accounts", skip_all)]
pub async fn get_external_accounts(
    State(app_state): State<AppState>,
) -> HttpJsonDatamizeResult<Vec<ExternalAccount>> {
    let db_conn_pool = app_state.db_conn_pool;

    let saved_accounts = get_all_external_accounts(&db_conn_pool)
        .await
        .map_err(AppError::from_sqlx)?
        .into_iter()
        .map(|wsa| wsa.into())
        .collect();

    Ok(Json(saved_accounts))
}
