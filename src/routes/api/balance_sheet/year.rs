use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    models::balance_sheet::{UpdateYear, YearDetail},
    services::balance_sheet::{YearService, YearServiceExt},
};

/// Returns a detailed year with its balance sheet, its saving rates, its months and its financial resources.
#[tracing::instrument(name = "Get a detailed year", skip_all)]
pub async fn balance_sheet_year(
    Path(year): Path<i32>,
    State(year_service): State<YearService>,
) -> HttpJsonDatamizeResult<YearDetail> {
    Ok(Json(year_service.get_year(year).await?))
}

/// Updates the saving rates of the received year.
#[tracing::instrument(skip_all)]
pub async fn update_balance_sheet_year(
    Path(year): Path<i32>,
    State(year_service): State<YearService>,
    WithRejection(Json(body), _): WithRejection<Json<UpdateYear>, JsonError>,
) -> HttpJsonDatamizeResult<YearDetail> {
    Ok(Json(year_service.update_year(year, body).await?))
}

/// Deletes the year and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_year(
    Path(year): Path<i32>,
    State(year_service): State<YearService>,
) -> HttpJsonDatamizeResult<YearDetail> {
    Ok(Json(year_service.delete_year(year).await?))
}
