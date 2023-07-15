use crate::{
    db::budget_template::get_all_budgeters_config,
    models::budget_template::{BudgetDetails, MonthQueryParam},
    routes::api::budget_template::common::{
        get_categories_of_month, get_latest_scheduled_transactions,
    },
};
use anyhow::Context;
use axum::{
    extract::{Query, State},
    Json,
};
use futures::try_join;

use crate::{error::HttpJsonAppResult, get_redis_conn, startup::AppState};

/// Returns a budget template details
/// Can specify the month to get details from.
/// /template/details?month=previous
/// Possible values to pass in query params are `previous` and `next`. If nothing is specified,
/// the current month will be used.
pub async fn template_details(
    State(app_state): State<AppState>,
    month: Option<Query<MonthQueryParam>>,
) -> HttpJsonAppResult<BudgetDetails> {
    let ynab_client = app_state.ynab_client.as_ref();
    let db_conn_pool = app_state.db_conn_pool;
    let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get redis connection from pool")?;
    let mut second_redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get second redis connection from pool")?;

    let Query(MonthQueryParam { month }) = month.unwrap_or_default();

    // TODO: Discard knowledge_server when changing month.
    let (saved_categories, saved_scheduled_transactions) = try_join!(
        get_categories_of_month(&db_conn_pool, &mut redis_conn, ynab_client, month),
        get_latest_scheduled_transactions(&db_conn_pool, &mut second_redis_conn, ynab_client)
    )
    .context("failed to get latest categories and scheduled transactions")?;
    let budgeters_config = get_all_budgeters_config(&db_conn_pool).await?;

    let data = BudgetDetails::build(
        saved_categories,
        saved_scheduled_transactions,
        &month.into(),
        (*app_state.budget_calculation_data_settings).clone(), // TODO: Get this from DB once it's possible for user to choose the payees in the frontend.
        budgeters_config,
    );

    Ok(Json(data))
}
