use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    db::balance_sheet::{delete_month, get_month},
    error::{AppError, HttpJsonAppResult},
    models::balance_sheet::{Month, MonthNum},
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
        get_month(&db_conn_pool, month, year)
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

    let month_detail = get_month(&db_conn_pool, month, year)
        .await
        .map_err(AppError::from_sqlx)?;
    delete_month(&db_conn_pool, month, year).await?;

    Ok(Json(month_detail))
}
