use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    models::balance_sheet::{FinancialResourceYearly, SaveResource},
    services::balance_sheet::{FinResService, FinResServiceExt},
};

/// Returns all resources of all years.
#[tracing::instrument(name = "Get all resources from all years", skip_all)]
pub async fn all_balance_sheet_resources(
    State(fin_res_service): State<FinResService>,
) -> HttpJsonDatamizeResult<Vec<FinancialResourceYearly>> {
    Ok(Json(fin_res_service.get_all_fin_res().await?))
}

#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_resource(
    State(fin_res_service): State<FinResService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveResource>, JsonError>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::CREATED,
        Json(fin_res_service.create_fin_res(body).await?),
    ))
}

/// Endpoint to get all financial resources of a particular year.
#[tracing::instrument(name = "Get all resources from a year", skip_all)]
pub async fn balance_sheet_resources(
    Path(year): Path<i32>,
    State(fin_res_service): State<FinResService>,
) -> HttpJsonDatamizeResult<Vec<FinancialResourceYearly>> {
    Ok(Json(fin_res_service.get_all_fin_res_from_year(year).await?))
}

// TODO: Check this https://github.com/tokio-rs/axum/blob/main/examples/testing/src/main.rs to unit test routes directly.
