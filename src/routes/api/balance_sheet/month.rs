use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    db,
    domain::{Month, MonthNum},
    error::{AppError, HttpJsonAppResult},
    startup::AppState,
};

/// Returns a specific month with its financial resources and net totals.
#[tracing::instrument(name = "Get a month", skip_all)]
pub async fn balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Month> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(
        db::get_month(&db_conn_pool, month, year)
            .await
            .map_err(AppError::from_sqlx)?,
    ))
}

/// Deletes the month and returns the entity
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Month> {
    let db_conn_pool = app_state.db_conn_pool;

    let month_detail = db::get_month(&db_conn_pool, month, year)
        .await
        .map_err(AppError::from_sqlx)?;
    db::delete_month(&db_conn_pool, month, year).await?;

    Ok(Json(month_detail))
}
