use axum::{
    extract::{Path, State},
    Json,
};
use datamize_domain::Year;

use crate::{error::HttpJsonDatamizeResult, services::balance_sheet::DynYearService};

/// Returns a detailed year with its balance sheet, its saving rates, its months and its financial resources.
#[tracing::instrument(name = "Get a detailed year", skip_all)]
pub async fn balance_sheet_year(
    Path(year): Path<i32>,
    State(year_service): State<DynYearService>,
) -> HttpJsonDatamizeResult<Year> {
    Ok(Json(year_service.get_year(year).await?))
}

/// Deletes the year and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_year(
    Path(year): Path<i32>,
    State(year_service): State<DynYearService>,
) -> HttpJsonDatamizeResult<Year> {
    Ok(Json(year_service.delete_year(year).await?))
}
