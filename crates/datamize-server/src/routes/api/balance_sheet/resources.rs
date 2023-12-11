#[cfg(test)]
mod create;
#[cfg(test)]
mod delete;
#[cfg(test)]
mod get;
#[cfg(test)]
mod get_all;
#[cfg(test)]
mod get_all_from_year;
#[cfg(test)]
pub(crate) mod testutils;
#[cfg(test)]
mod update;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;
use datamize_domain::{FinancialResourceYearly, SaveResource};

use crate::{
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    services::balance_sheet::DynFinResService,
};

/// Returns all resources of all years.
#[tracing::instrument(name = "Get all resources from all years", skip_all)]
pub async fn all_balance_sheet_resources(
    State(fin_res_service): State<DynFinResService>,
) -> HttpJsonDatamizeResult<Vec<FinancialResourceYearly>> {
    Ok(Json(fin_res_service.get_all_fin_res().await?))
}

/// Endpoint to get all financial resources of a particular year.
#[tracing::instrument(name = "Get all resources from a year", skip_all)]
pub async fn balance_sheet_resources(
    Path(year): Path<i32>,
    State(fin_res_service): State<DynFinResService>,
) -> HttpJsonDatamizeResult<Vec<FinancialResourceYearly>> {
    Ok(Json(fin_res_service.get_all_fin_res_from_year(year).await?))
}

#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_resource(
    State(fin_res_service): State<DynFinResService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveResource>, JsonError>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::CREATED,
        Json(fin_res_service.create_fin_res(body).await?),
    ))
}
