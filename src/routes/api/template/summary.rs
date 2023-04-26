use anyhow::Context;
use axum::{extract::State, Json};
use futures::try_join;

use crate::{
    error::HttpJsonAppResult,
    get_redis_conn,
    models::budget_template::CommonExpenseEstimationPerPerson,
    routes::api::template::common::{get_latest_categories, get_latest_scheduled_transactions},
    startup::AppState,
};

use super::common::{build_budget_details, build_budget_summary};

/// Returns a budget template summary.
pub async fn template_summary(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<CommonExpenseEstimationPerPerson>> {
    let ynab_client = app_state.ynab_client.as_ref();
    let db_conn_pool = app_state.db_conn_pool;
    let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get redis connection from pool")?;
    let mut second_redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get second redis connection from pool")?;

    let (saved_categories, saved_scheduled_transactions) = try_join!(
        get_latest_categories(&db_conn_pool, &mut redis_conn, ynab_client),
        get_latest_scheduled_transactions(&db_conn_pool, &mut second_redis_conn, ynab_client)
    )
    .context("failed to get latest categories and scheduled transactions")?;

    let budget_details = build_budget_details(
        saved_categories,
        saved_scheduled_transactions.clone(),
        &app_state.budget_calculation_data_settings,
        &app_state.person_salary_settings,
    )
    .context("failed to compute budget details")?;

    let data = build_budget_summary(
        &budget_details,
        &saved_scheduled_transactions,
        &app_state.person_salary_settings,
    )
    .context("failed to compute budget summary")?;

    Ok(Json(data))
}
