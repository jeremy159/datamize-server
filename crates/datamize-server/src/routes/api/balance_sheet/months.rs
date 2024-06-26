use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use datamize_domain::{Month, SaveMonth};

use crate::{
    error::{AppError, AppJson, HttpJsonDatamizeResult},
    services::balance_sheet::DynMonthService,
};

/// Returns all months of all years.
#[tracing::instrument(name = "Get all months from all years", skip_all)]
pub async fn all_balance_sheet_months(
    State(month_service): State<DynMonthService>,
) -> HttpJsonDatamizeResult<Vec<Month>> {
    Ok(AppJson(month_service.get_all_months().await?))
}

/// Returns all the months within a year with balance sheets.
#[tracing::instrument(name = "Get all months from a year", skip_all)]
pub async fn balance_sheet_months(
    Path(year): Path<i32>,
    State(month_service): State<DynMonthService>,
) -> HttpJsonDatamizeResult<Vec<Month>> {
    Ok(AppJson(month_service.get_months_from_year(year).await?))
}

/// Creates a new month if it doesn't already exist and returns the newly created entity.
/// Will also update net totals for this month compared to previous one if any.
#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_month(
    Path(year): Path<i32>,
    State(month_service): State<DynMonthService>,
    AppJson(body): AppJson<SaveMonth>,
) -> impl IntoResponse {
    Ok::<_, AppError>((
        StatusCode::CREATED,
        AppJson(month_service.create_month(year, body).await?),
    ))
}
