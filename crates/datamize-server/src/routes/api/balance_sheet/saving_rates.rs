#[cfg(test)]
mod tests;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    models::balance_sheet::{SaveSavingRate, SavingRate},
    services::balance_sheet::DynSavingRateService,
};

/// Endpoint to get all Saving Rates of a particular year.
#[tracing::instrument(name = "Get all saving rates from a year", skip_all)]
pub async fn balance_sheet_saving_rates(
    Path(year): Path<i32>,
    State(mut saving_rate_service): State<DynSavingRateService>,
) -> HttpJsonDatamizeResult<Vec<SavingRate>> {
    Ok(Json(saving_rate_service.get_all_from_year(year).await?))
}

#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_saving_rate(
    State(mut saving_rate_service): State<DynSavingRateService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveSavingRate>, JsonError>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::CREATED,
        Json(saving_rate_service.create_saving_rate(body).await?),
    ))
}

// TODO: Introduce SQLite for unit tests. See https://jmmv.dev/2023/07/unit-testing-a-web-service.html
// To introduce SQLite, need to split db implementations into their own crates, so making use of cargo workspace. https://github.com/launchbadge/sqlx/issues/121
// TODO: For better tests, see https://matklad.github.io/2021/05/31/how-to-test.html