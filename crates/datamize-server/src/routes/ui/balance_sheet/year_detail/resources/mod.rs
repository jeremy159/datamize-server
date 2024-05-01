pub mod total;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use axum_extra::extract::Form;
use datamize_domain::{FinancialResourceYearly, ResourceCategory, Uuid, YearlyBalances};
use serde::Deserialize;

use crate::{
    error::DatamizeResult,
    routes::ui::{num_to_currency, num_to_currency_rounded},
    services::balance_sheet::DynFinResService,
};

pub async fn get(
    Path((year, category)): Path<(i32, ResourceCategory)>,
    State(fin_res_service): State<DynFinResService>,
) -> DatamizeResult<impl IntoResponse> {
    let resources = fin_res_service
        .get_from_year_and_category(year, &category)
        .await?;

    Ok(ResourceRowsTemplate {
        year,
        category,
        resources,
    })
}

/// To sort resources
pub async fn post(
    Path((year, category)): Path<(i32, ResourceCategory)>,
    State(fin_res_service): State<DynFinResService>,
    Form(payload): Form<Payload>,
) -> DatamizeResult<impl IntoResponse> {
    fin_res_service
        .save_resources_order(year, &category, &payload.fin_res_ids)
        .await?;

    let resources = fin_res_service
        .get_from_year_and_category(year, &category)
        .await?;

    Ok(ResourceRowsTemplate {
        year,
        category,
        resources,
    })
}

#[derive(Deserialize)]
pub struct Payload {
    #[serde(rename = "fin_res_id")]
    fin_res_ids: Vec<Uuid>,
}

#[derive(Template)]
#[template(path = "partials/year-details/resource-rows.html")]
struct ResourceRowsTemplate {
    year: i32,
    category: ResourceCategory,
    resources: Vec<FinancialResourceYearly>,
}
