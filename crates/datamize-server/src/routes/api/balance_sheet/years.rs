#[cfg(test)]
mod create;
#[cfg(test)]
mod delete;
#[cfg(test)]
mod get;
#[cfg(test)]
mod get_all;
#[cfg(test)]
pub(crate) mod testutils;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;
use datamize_domain::{SaveYear, Year};

use crate::{
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    services::balance_sheet::DynYearService,
};

/// Returns a summary of all the years with balance sheets.
#[tracing::instrument(name = "Get all years", skip_all)]
pub async fn balance_sheet_years(
    State(year_service): State<DynYearService>,
) -> HttpJsonDatamizeResult<Vec<Year>> {
    Ok(Json(year_service.get_all_years().await?))
}

/// Creates a new year if it doesn't already exist and returns the newly created entity.
/// Will also update net totals for this year compared to previous one if any.
#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_year(
    State(year_service): State<DynYearService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveYear>, JsonError>,
) -> impl IntoResponse {
    Ok::<_, AppError>((
        StatusCode::CREATED,
        Json(year_service.create_year(body).await?),
    ))
}
