use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Query, State};
use datamize_domain::{get_res_type_options, ResourceCategoryQuery, ResourceTypeOption};

use crate::{
    error::DatamizeResult,
    services::{
        balance_sheet::{DynFinResService, DynYearService},
        budget_providers::{DynExternalAccountService, DynYnabAccountService},
    },
};

pub async fn get(
    Query(param): Query<ResourceCategoryQuery>,
    State((fin_res_service, _, _, _)): State<(
        DynFinResService,
        DynYnabAccountService,
        DynExternalAccountService,
        DynYearService,
    )>,
) -> DatamizeResult<impl IntoResponse> {
    let fin_res = match param.fin_res_id {
        Some(id) => Some(fin_res_service.get_fin_res(id).await?),
        None => None,
    };

    let resource_types: Vec<ResourceTypeOption> =
        get_res_type_options(param.category, &fin_res.map(|r| r.base.resource_type));

    Ok(ResourceTypeOptionsTemplate { resource_types })
}

#[derive(Template)]
#[template(path = "partials/financial-resource/types.html")]
struct ResourceTypeOptionsTemplate {
    resource_types: Vec<ResourceTypeOption>,
}
