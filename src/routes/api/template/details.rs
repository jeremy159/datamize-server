use crate::{
    models::budget_template::BudgetDetails,
    routes::api::template::common::{get_latest_categories, get_latest_scheduled_transactions},
};
use anyhow::Context;
use axum::{extract::State, Json};
use futures::try_join;

use crate::{error::HttpJsonAppResult, get_redis_conn, startup::AppState};

use super::common::build_budget_details;

// TODO: Implement template details for prev, current and next months
/// Returns a budget template details
pub async fn template_details(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<BudgetDetails> {
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

    let data = build_budget_details(
        saved_categories,
        saved_scheduled_transactions,
        &app_state.budget_calculation_data_settings,
        &app_state.person_salary_settings,
    )
    .context("failed to compute budget details")?;

    Ok(Json(data))
}
