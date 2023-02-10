use axum::{
    extract::{Path, State},
    Json,
};
use ynab::types::Account;

use crate::{error::HttpJsonAppResult, startup::AppState};

/// Returns a detailed year with its balance sheet and its saving rates.
pub async fn balance_sheet_year(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Account>> {
    let ynab_client = app_state.ynab_client.as_ref();
    let data = ynab_client.get_accounts().await?;
    Ok(Json(data))
}
