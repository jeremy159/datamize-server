use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    error::HttpJsonDatamizeResult,
    models::balance_sheet::{Month, MonthNum},
    services::balance_sheet::{MonthService, MonthServiceExt},
};

/// Returns a specific month with its financial resources and net totals.
#[tracing::instrument(name = "Get a month", skip_all)]
pub async fn balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(month_service): State<MonthService>,
) -> HttpJsonDatamizeResult<Month> {
    Ok(Json(month_service.get_month(month, year).await?))
}

/// Deletes the month and returns the entity
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(month_service): State<MonthService>,
) -> HttpJsonDatamizeResult<Month> {
    Ok(Json(month_service.delete_month(month, year).await?))
}
