use redis::Connection;
use sqlx::PgPool;

use anyhow::Context;
use ynab::types::Payee;

use crate::db;
use crate::db::budget_providers::ynab::{get_payees_delta, save_payees, set_payees_detla};
use crate::error::AppError;

pub async fn get_payees(
    ynab_client: &ynab::Client,
    db_conn_pool: &PgPool,
    redis_conn: &mut Connection,
) -> Result<Vec<Payee>, AppError> {
    let saved_payees_delta = get_payees_delta(redis_conn);

    let payees_delta = ynab_client
        .get_payees_delta(saved_payees_delta)
        .await
        .context("failed to get payees from ynab's API")?;

    let payees = payees_delta
        .payees
        .into_iter()
        .filter(|a| !a.deleted)
        .collect::<Vec<_>>();

    save_payees(db_conn_pool, &payees)
        .await
        .context("failed to save payees in database")?;

    set_payees_detla(redis_conn, payees_delta.server_knowledge)
        .context("failed to save last known server knowledge of payees in redis")?;

    let saved_payees = db::budget_providers::ynab::get_payees(db_conn_pool)
        .await
        .context("failed to get payees from database")?;

    Ok(saved_payees)
}
