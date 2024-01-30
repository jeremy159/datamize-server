use axum::{
    extract::{rejection::JsonRejection, State},
    Json,
};
use datamize_domain::{ResourcesToRefresh, Uuid};

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    services::balance_sheet::DynRefreshFinResService,
};

/// Endpoint to refresh financial resources.
/// Only resources from the current month will be refreshed by this endpoint.
/// If current month does not exists, it will create it.
/// An optionnal body can be passed to specify which resources to refresh.
///
/// This endpoint basically calls the YNAB api for some resources and starts a web scrapper for others.
/// Will return an array of ids for Financial Resources updated.
#[tracing::instrument(skip_all)]
pub async fn refresh_balance_sheet_resources(
    State(fin_res_service): State<DynRefreshFinResService>,
    payload: Result<Json<ResourcesToRefresh>, JsonRejection>,
) -> HttpJsonDatamizeResult<Vec<Uuid>> {
    println!("{payload:#?}");
    let body = match payload {
        Ok(p) => Some(p.0),
        Err(
            JsonRejection::MissingJsonContentType(_)
            | JsonRejection::JsonSyntaxError(_)
            | JsonRejection::BytesRejection(_),
        ) => return Err(Into::<JsonError>::into(payload.err().unwrap()))?,
        Err(_) => None,
    };
    Ok(Json(fin_res_service.refresh_fin_res(body).await?))
}
