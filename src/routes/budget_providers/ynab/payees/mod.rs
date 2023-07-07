use anyhow::Context;
use axum::{extract::State, Json};
use ynab::types::Payee;

use crate::error::HttpJsonAppResult;
use crate::get_redis_conn;
use crate::startup::AppState;

use self::utils::get_payees;

mod utils;

/// Returns all accounts from YNAB's API.
#[tracing::instrument(name = "Get all payees from YNAB's API", skip_all)]
pub async fn get_ynab_payees(State(app_state): State<AppState>) -> HttpJsonAppResult<Vec<Payee>> {
    let ynab_client = app_state.ynab_client.as_ref();
    let db_conn_pool = app_state.db_conn_pool;
    let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get redis connection from pool")?;

    let saved_payees = get_payees(ynab_client, &db_conn_pool, &mut redis_conn)
        .await
        .context("failed to get payees")?;

    Ok(Json(saved_payees))
}
