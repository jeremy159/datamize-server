use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;
use datamize_domain::{SaveSavingRate, SavingRate};

use crate::{
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    services::balance_sheet::DynSavingRateService,
};

/// Endpoint to get all Saving Rates of a particular year.
#[tracing::instrument(name = "Get all saving rates from a year", skip_all)]
pub async fn balance_sheet_saving_rates(
    Path(year): Path<i32>,
    State(saving_rate_service): State<DynSavingRateService>,
) -> HttpJsonDatamizeResult<Vec<SavingRate>> {
    Ok(Json(saving_rate_service.get_all_from_year(year).await?))
}

#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_saving_rate(
    State(saving_rate_service): State<DynSavingRateService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveSavingRate>, JsonError>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::CREATED,
        Json(saving_rate_service.create_saving_rate(body).await?),
    ))
}
