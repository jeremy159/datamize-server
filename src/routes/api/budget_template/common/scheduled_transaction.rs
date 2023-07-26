use anyhow::Context;
use chrono::{Datelike, Local, NaiveDate};

use crate::{
    db::budget_providers::ynab::{
        del_scheduled_transactions_delta, get_scheduled_transactions,
        get_scheduled_transactions_delta, get_scheduled_transactions_last_saved,
        save_scheduled_transactions, set_scheduled_transactions_delta,
        set_scheduled_transactions_last_saved,
    },
    models::budget_template::DatamizeScheduledTransaction,
};

pub async fn get_latest_scheduled_transactions(
    db_conn_pool: &sqlx::PgPool,
    redis_conn: &mut redis::Connection,
    ynab_client: &ynab::Client,
) -> anyhow::Result<Vec<DatamizeScheduledTransaction>> {
    let current_date = Local::now().date_naive();
    if let Some(last_saved) = get_scheduled_transactions_last_saved(redis_conn) {
        let last_saved_date: NaiveDate = last_saved.parse()?;
        if current_date.month() != last_saved_date.month() {
            // Discard knowledge_server when changing month.
            del_scheduled_transactions_delta(redis_conn)?;
            set_scheduled_transactions_last_saved(redis_conn, current_date.to_string())?;
        }
    } else {
        set_scheduled_transactions_last_saved(redis_conn, current_date.to_string())?;
    }
    let saved_scheduled_transactions_delta = get_scheduled_transactions_delta(redis_conn);

    let scheduled_transactions_delta = ynab_client
        .get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
        .await
        .context("failed to get scheduled transactions from ynab's API")?;

    save_scheduled_transactions(
        db_conn_pool,
        &scheduled_transactions_delta.scheduled_transactions,
    )
    .await
    .context("failed to save scheduled transactions in database")?;

    set_scheduled_transactions_delta(redis_conn, scheduled_transactions_delta.server_knowledge)
        .context("failed to save last known server knowledge of scheduled transactions in redis")?;

    Ok(get_scheduled_transactions(db_conn_pool)
        .await
        .context("failed to get scheduled transactions from database")?
        .into_iter()
        .map(Into::into)
        .collect())
}
