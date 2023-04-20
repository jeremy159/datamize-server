use std::collections::HashMap;

use anyhow::Context;
use axum::{extract::State, Json};
use budget_data_api::{CategoryIdToNameMap, ScheduledTransactionsDistribution};
use futures::{stream::FuturesUnordered, StreamExt};

use crate::{
    db::{
        get_category_by_id, get_scheduled_transactions, get_scheduled_transactions_delta,
        save_scheduled_transactions, set_scheduled_transactions_delta,
    },
    error::HttpJsonAppResult,
    get_redis_conn,
    startup::AppState,
};

/// Returns a budget template transactions, i.e. all the scheduled transactions in the upcoming month.
pub async fn template_transactions(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<ScheduledTransactionsDistribution> {
    let ynab_client = app_state.ynab_client.as_ref();
    let db_conn_pool = app_state.db_conn_pool;
    let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get redis connection from pool")?;

    let saved_scheduled_transactions_delta = get_scheduled_transactions_delta(&mut redis_conn);

    let scheduled_transactions_delta = ynab_client
        .get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
        .await
        .context("failed to get scheduled transactions from ynab's API")?;

    save_scheduled_transactions(
        &db_conn_pool,
        &scheduled_transactions_delta.scheduled_transactions,
    )
    .await
    .context("failed to save scheduled transactions in database")?;

    set_scheduled_transactions_delta(
        &mut redis_conn,
        scheduled_transactions_delta.server_knowledge,
    )
    .context("failed to save last known server knowledge of scheduled transactions in redis")?;

    let saved_scheduled_transactions = get_scheduled_transactions(&db_conn_pool)
        .await
        .context("failed to get scheduled transactions from database")?;

    let category_ids =
        budget_data_api::get_subtransactions_category_ids(&saved_scheduled_transactions);

    let mut category_id_to_name_map: CategoryIdToNameMap = HashMap::new();

    let categories_stream = category_ids
        .iter()
        .map(|cat_id| get_category_by_id(&db_conn_pool, cat_id))
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await;

    for (index, category) in categories_stream.into_iter().enumerate() {
        let category = match category.context(format!(
            "failed to find category {} in database",
            category_ids[index]
        ))? {
            Some(cat) => cat,
            None => ynab_client
                .get_category_by_id(&category_ids[index].to_string())
                .await
                .context(format!(
                    "failed to get category {} in ynab",
                    category_ids[index]
                ))?,
        };
        category_id_to_name_map.insert(category.id, category.name);
    }

    let data = budget_data_api::build_scheduled_transactions(
        &saved_scheduled_transactions,
        &category_id_to_name_map,
    )
    .context("failed to compute scheduled transactions map")?;

    Ok(Json(data))
}
