use axum::extract::State;
use axum_extra::extract::OptionalQuery;
use datamize_domain::{ResourcesToRefresh, Uuid};

use crate::{
    error::{AppJson, HttpJsonDatamizeResult},
    services::balance_sheet::DynRefreshFinResService,
};

/// Endpoint to refresh financial resources.
/// Only resources from the current month will be refreshed by this endpoint.
/// If current month does not exists, it will create it.
/// An optionnal query parameter can be passed to specify which resources to refresh.
///
/// This endpoint basically calls the YNAB api for some resources and starts a web scrapper for others.
/// Will return an array of ids for Financial Resources updated.
#[tracing::instrument(skip_all)]
pub async fn refresh_balance_sheet_resources(
    State(fin_res_service): State<DynRefreshFinResService>,
    OptionalQuery(params): OptionalQuery<ResourcesToRefresh>,
) -> HttpJsonDatamizeResult<Vec<Uuid>> {
    Ok(AppJson(fin_res_service.refresh_fin_res(params).await?))
}
