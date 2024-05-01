use askama_axum::IntoResponse;
use axum::{extract::State, response::Html};
use axum_extra::extract::OptionalQuery;
use datamize_domain::ResourcesToRefresh;

use crate::{error::DatamizeResult, services::balance_sheet::DynRefreshFinResService};

pub async fn post(
    State(fin_res_service): State<DynRefreshFinResService>,
    OptionalQuery(params): OptionalQuery<ResourcesToRefresh>,
) -> DatamizeResult<impl IntoResponse> {
    Ok(match fin_res_service.refresh_fin_res(params).await {
        Ok(ids) => {
            if ids.is_empty() {
                Html("").into_response()
            } else {
                ([("Hx-Trigger", "resources-refreshed")], Html("")).into_response()
            }
        }
        Err(_) => Html("").into_response(),
    })
}
