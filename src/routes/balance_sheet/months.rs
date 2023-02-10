use axum::{extract::State, Json};
use ynab::types::Account;

use crate::{error::HttpJsonAppResult, startup::AppState};

pub async fn balance_sheet_months(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Account>> {
    let ynab_client = app_state.ynab_client.as_ref();
    let data = ynab_client.get_accounts().await?;
    Ok(Json(data))
}
