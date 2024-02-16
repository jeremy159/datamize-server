use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use datamize_domain::{SaveSavingRate, SavingRate};

use crate::{
    error::{AppError, AppJson, HttpJsonDatamizeResult},
    services::balance_sheet::DynSavingRateService,
};

/// Endpoint to get all Saving Rates of a particular year.
#[tracing::instrument(name = "Get all saving rates from a year", skip_all)]
pub async fn balance_sheet_saving_rates(
    Path(year): Path<i32>,
    State(saving_rate_service): State<DynSavingRateService>,
) -> HttpJsonDatamizeResult<Vec<SavingRate>> {
    Ok(AppJson(saving_rate_service.get_all_from_year(year).await?))
}

#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_saving_rate(
    State(saving_rate_service): State<DynSavingRateService>,
    AppJson(body): AppJson<SaveSavingRate>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::CREATED,
        AppJson(saving_rate_service.create_saving_rate(body).await?),
    ))
}
