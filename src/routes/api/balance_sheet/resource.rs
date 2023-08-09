use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    models::balance_sheet::{FinancialResourceYearly, SaveResource},
    services::balance_sheet::{FinResService, FinResServiceExt},
};

/// Returns a specific resource.
#[tracing::instrument(name = "Get a resource", skip_all)]
pub async fn balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(fin_res_service): State<FinResService>,
) -> HttpJsonDatamizeResult<FinancialResourceYearly> {
    Ok(Json(fin_res_service.get_fin_res(resource_id).await?))
}

/// Updates the resource. Will create any non-existing months.
/// Will also update the months' and year's net totals.
#[tracing::instrument(skip_all)]
pub async fn update_balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(fin_res_service): State<FinResService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveResource>, JsonError>,
) -> HttpJsonDatamizeResult<FinancialResourceYearly> {
    Ok(Json(
        fin_res_service.update_fin_res(resource_id, body).await?,
    ))
}

/// Deletes the resource and returns the entity
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(fin_res_service): State<FinResService>,
) -> HttpJsonDatamizeResult<FinancialResourceYearly> {
    Ok(Json(fin_res_service.delete_fin_res(resource_id).await?))
}
