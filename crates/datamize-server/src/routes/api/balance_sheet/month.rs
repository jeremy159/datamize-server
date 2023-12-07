use axum::{
    extract::{Path, State},
    Json,
};
use datamize_domain::{Month, MonthNum};

use crate::{error::HttpJsonDatamizeResult, services::balance_sheet::DynMonthService};

/// Returns a specific month with its financial resources and net totals.
#[tracing::instrument(name = "Get a month", skip_all)]
pub async fn balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(month_service): State<DynMonthService>,
) -> HttpJsonDatamizeResult<Month> {
    Ok(Json(month_service.get_month(month, year).await?))
}

/// Deletes the month and returns the entity
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(month_service): State<DynMonthService>,
) -> HttpJsonDatamizeResult<Month> {
    Ok(Json(month_service.delete_month(month, year).await?))
}
