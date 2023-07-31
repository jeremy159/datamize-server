use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;

use crate::{
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    models::balance_sheet::{SaveYear, YearSummary},
    services::balance_sheet::YearServiceExt,
};

/// Returns a summary of all the years with balance sheets.
#[tracing::instrument(name = "Get a summary of all years", skip_all)]
pub async fn balance_sheet_years<YS: YearServiceExt>(
    State(year_service): State<YS>,
) -> HttpJsonDatamizeResult<Vec<YearSummary>> {
    Ok(Json(year_service.get_all_years().await?))
}

/// Creates a new year if it doesn't already exist and returns the newly created entity.
/// Will also update net totals for this year compared to previous one if any.
#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_year<YS: YearServiceExt>(
    State(year_service): State<YS>,
    WithRejection(Json(body), _): WithRejection<Json<SaveYear>, JsonError>,
) -> impl IntoResponse {
    Ok::<_, AppError>((
        StatusCode::CREATED,
        Json(year_service.create_year(body).await?),
    ))
}
