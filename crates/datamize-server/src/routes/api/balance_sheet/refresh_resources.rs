use axum::{extract::State, Json};
use datamize_domain::Uuid;

use crate::{error::HttpJsonDatamizeResult, services::balance_sheet::DynRefreshFinResService};

/// Endpoint to refresh financial resources.
/// Only resources from the current month will be refreshed by this endpoint.
/// If current month does not exists, it will create it.
/// This endpoint basically calls the YNAB api for some resources and starts a web scrapper for others.
/// Will return an array of ids for Financial Resources updated.
// TODO: Add ability to specify which resources to refresh.
#[tracing::instrument(skip_all)]
pub async fn refresh_balance_sheet_resources(
    State(mut fin_res_service): State<DynRefreshFinResService>,
) -> HttpJsonDatamizeResult<Vec<Uuid>> {
    Ok(Json(fin_res_service.refresh_fin_res().await?))
}
