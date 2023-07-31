use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    models::balance_sheet::{Month, SaveMonth},
    services::balance_sheet::MonthServiceExt,
};

/// Returns all months of all years.
#[tracing::instrument(name = "Get all months from all years", skip_all)]
pub async fn all_balance_sheet_months<MS: MonthServiceExt>(
    State(month_service): State<MS>,
) -> HttpJsonDatamizeResult<Vec<Month>> {
    Ok(Json(month_service.get_all_months().await?))
}

/// Returns all the months within a year with balance sheets.
#[tracing::instrument(name = "Get all months from a year", skip_all)]
pub async fn balance_sheet_months<MS: MonthServiceExt>(
    Path(year): Path<i32>,
    State(month_service): State<MS>,
) -> HttpJsonDatamizeResult<Vec<Month>> {
    Ok(Json(month_service.get_all_months_from_year(year).await?))
}

/// Creates a new month if it doesn't already exist and returns the newly created entity.
/// Will also update net totals for this month compared to previous one if any.
#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_month<MS: MonthServiceExt>(
    Path(year): Path<i32>,
    State(month_service): State<MS>,
    WithRejection(Json(body), _): WithRejection<Json<SaveMonth>, JsonError>,
) -> impl IntoResponse {
    Ok::<_, AppError>((
        StatusCode::CREATED,
        Json(month_service.create_month(year, body).await?),
    ))
}
