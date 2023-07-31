use crate::{
    db::budget_template::{get_all_budgeters_config, get_all_external_expenses},
    models::budget_template::{BudgetDetails, Budgeter, Configured, MonthQueryParam},
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

use crate::{error::HttpJsonDatamizeResult, get_redis_conn, startup::AppState};

/// Returns a budget template details
/// Can specify the month to get details from.
/// /template/details?month=previous
/// Possible values to pass in query params are `previous` and `next`. If nothing is specified,
/// the current month will be used.
pub async fn template_details(
    State(app_state): State<AppState>,
    month: Option<Query<MonthQueryParam>>,
) -> HttpJsonDatamizeResult<BudgetDetails> {
    let ynab_client = app_state.ynab_client.as_ref();
    let db_conn_pool = app_state.db_conn_pool;
    let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get redis connection from pool")?;
    let mut second_redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get second redis connection from pool")?;

    let Query(MonthQueryParam(month)) = month.unwrap_or_default();

    let ((saved_categories, expenses_categorization), saved_scheduled_transactions) = try_join!(
        get_categories_of_month(&db_conn_pool, &mut redis_conn, ynab_client, month),
        get_latest_scheduled_transactions(&db_conn_pool, &mut second_redis_conn, ynab_client)
    )
    .context("failed to get latest categories and scheduled transactions")?;
    let external_expenses = get_all_external_expenses(&db_conn_pool).await?;
    let budgeters_config = get_all_budgeters_config(&db_conn_pool).await?;
    let budgeters: Vec<_> = budgeters_config
        .into_iter()
        .map(|bc| Budgeter::<Configured>::from(bc).compute_salary(&saved_scheduled_transactions))
        .collect();

    let data = BudgetDetails::build(
        saved_categories,
        saved_scheduled_transactions,
        &month.into(),
        external_expenses,
        expenses_categorization,
        &budgeters,
    );

    Ok(Json(data))
}
