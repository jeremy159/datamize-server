use std::collections::HashMap;

use anyhow::Context;
use axum::{extract::State, Json};
use futures::{stream::FuturesUnordered, StreamExt};

use crate::{
    db::budget_providers::ynab::*,
    error::HttpJsonDatamizeResult,
    get_redis_conn,
    models::budget_template::{
        CategoryIdToNameMap, DatamizeScheduledTransaction, ScheduledTransactionsDistribution,
    },
    startup::AppState,
};

use super::common::get_latest_scheduled_transactions;

/// Returns a budget template transactions, i.e. all the scheduled transactions in the upcoming 30 days.
pub async fn template_transactions(
    State(app_state): State<AppState>,
) -> HttpJsonDatamizeResult<ScheduledTransactionsDistribution> {
    let ynab_client = app_state.ynab_client.as_ref();
    let db_conn_pool = app_state.db_conn_pool;
    let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get redis connection from pool")?;

    let saved_scheduled_transactions =
        get_latest_scheduled_transactions(&db_conn_pool, &mut redis_conn, ynab_client).await?;

    let category_ids = get_subtransactions_category_ids(&saved_scheduled_transactions);

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

    let data = ScheduledTransactionsDistribution::builder(saved_scheduled_transactions)
        .with_category_map(category_id_to_name_map)
        .build();

    Ok(Json(data))
}

fn get_subtransactions_category_ids(
    scheduled_transactions: &[DatamizeScheduledTransaction],
) -> Vec<uuid::Uuid> {
    scheduled_transactions
        .iter()
        .flat_map(|st| &st.subtransactions)
        .filter_map(|sub_st| sub_st.category_id)
        .collect()
}
