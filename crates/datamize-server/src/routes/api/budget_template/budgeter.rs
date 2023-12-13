use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use datamize_domain::{BudgeterConfig, Uuid};

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    services::budget_template::DynBudgeterService,
};

/// Returns a budgeter's config.
#[tracing::instrument(skip_all)]
pub async fn get_budgeter(
    Path(id): Path<Uuid>,
    State(budgeter_service): State<DynBudgeterService>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(Json(budgeter_service.get_budgeter(id).await?))
}

/// Updates the budgeter's name and payee_ids.
#[tracing::instrument(skip_all)]
pub async fn update_budgeter(
    Path(_id): Path<Uuid>,
    State(budgeter_service): State<DynBudgeterService>,
    WithRejection(Json(body), _): WithRejection<Json<BudgeterConfig>, JsonError>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(Json(budgeter_service.update_budgeter(body).await?))
}

/// Deletes the budgeter and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_budgeter(
    Path(id): Path<Uuid>,
    State(budgeter_service): State<DynBudgeterService>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(Json(budgeter_service.delete_budgeter(id).await?))
}
