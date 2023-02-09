use anyhow::Context;
use axum::{extract::State, Json};
use budget_data_api::CommonExpenseEstimationPerPerson;
use futures::try_join;

use crate::{
    db::{
        get_categories, get_categories_delta, get_scheduled_transactions,
        get_scheduled_transactions_delta, save_categories, save_scheduled_transactions,
        set_categories_detla, set_scheduled_transactions_delta,
    },
    error::HttpJsonAppResult,
    startup::{get_redis_conn, AppState},
};

/// Returns a budget template summary.
pub async fn template_summary(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<CommonExpenseEstimationPerPerson>> {
    let ynab_client = app_state.ynab_client.as_ref();
    let db_conn_pool = app_state.db_conn_pool;
    let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get redis connection from pool")?;

    let saved_categories_delta = get_categories_delta(&mut redis_conn);
    let saved_scheduled_transactions_delta = get_scheduled_transactions_delta(&mut redis_conn);

    let (category_groups_with_categories_delta, scheduled_transactions_delta) = try_join!(
        ynab_client.get_categories_delta(saved_categories_delta),
        ynab_client.get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
    )
    .context("failed to get categories or scheduled transactions from ynab's API")?;

    let categories = category_groups_with_categories_delta
        .category_groups
        .into_iter()
        .flat_map(|cg| cg.categories)
        .collect::<Vec<_>>();

    save_categories(&db_conn_pool, &categories)
        .await
        .context("failed to save categories in database")?;

    set_categories_detla(
        &mut redis_conn,
        category_groups_with_categories_delta.server_knowledge,
    )
    .context("failed to save last known server knowledge of categories in redis")?;

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

    let (saved_categories, saved_scheduled_transactions) = try_join!(
        get_categories(&db_conn_pool),
        get_scheduled_transactions(&db_conn_pool)
    )
    .context("failed to get categories and scheduled transactions from database")?;

    let data =
        budget_data_api::build_budget_summary(&saved_categories, &saved_scheduled_transactions)
            .context("failed to compute budget summary")?;

    Ok(Json(data))
}
