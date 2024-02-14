use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use datamize_domain::{FinancialResourceYearly, UpdateResource, Uuid};

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    services::balance_sheet::DynFinResService,
};

/// Returns a specific resource.
#[tracing::instrument(name = "Get a resource", skip_all)]
pub async fn balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(fin_res_service): State<DynFinResService>,
) -> HttpJsonDatamizeResult<FinancialResourceYearly> {
    Ok(Json(fin_res_service.get_fin_res(resource_id).await?))
}

/// Updates the resource. Will create any non-existing months.
/// Will also update the months' and year's net totals.
#[tracing::instrument(skip_all)]
pub async fn update_balance_sheet_resource(
    Path(_): Path<Uuid>,
    State(fin_res_service): State<DynFinResService>,
    WithRejection(Json(body), _): WithRejection<Json<UpdateResource>, JsonError>,
) -> HttpJsonDatamizeResult<FinancialResourceYearly> {
    Ok(Json(fin_res_service.update_fin_res(body).await?))
}

/// Deletes the resource and returns the entity
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(fin_res_service): State<DynFinResService>,
) -> HttpJsonDatamizeResult<FinancialResourceYearly> {
    Ok(Json(fin_res_service.delete_fin_res(resource_id).await?))
}
