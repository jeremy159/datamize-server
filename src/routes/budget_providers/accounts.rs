use anyhow::Context;
use axum::{extract::State, Json};
use ynab::types::Account;

use crate::db::{self, get_accounts_delta, set_accounts_detla};
use crate::error::HttpJsonAppResult;
use crate::startup::{get_redis_conn, AppState};

/// Returns all accounts from YNAB's API.
#[tracing::instrument(name = "Get all accounts from YNAB's API", skip_all)]
pub async fn get_ynab_accounts(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Account>> {
    let ynab_client = app_state.ynab_client.as_ref();
    let db_conn_pool = app_state.db_conn_pool;
    let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get redis connection from pool")?;

    let saved_accounts_delta = get_accounts_delta(&mut redis_conn);

    let accounts_delta = ynab_client
        .get_accounts_delta(saved_accounts_delta)
        .await
        .context("failed to get accounts from ynab's API")?;

    let accounts = accounts_delta
        .accounts
        .into_iter()
        .filter(|a| !a.deleted)
        .collect::<Vec<_>>();

    db::save_accounts(&db_conn_pool, &accounts)
        .await
        .context("failed to save accounts in database")?;

    set_accounts_detla(&mut redis_conn, accounts_delta.server_knowledge)
        .context("failed to save last known server knowledge of accounts in redis")?;

    let saved_accounts = db::get_accounts(&db_conn_pool)
        .await
        .context("failed to get accounts from database")?;

    Ok(Json(saved_accounts))
}
