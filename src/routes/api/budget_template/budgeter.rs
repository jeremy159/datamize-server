use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    models::budget_template::BudgeterConfig,
    services::budget_template::BudgeterServiceExt,
};

/// Returns a budgeter's config.
#[tracing::instrument(skip_all)]
pub async fn get_budgeter<BS: BudgeterServiceExt>(
    Path(id): Path<Uuid>,
    State(budgeter_service): State<BS>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(Json(budgeter_service.get_budgeter(id).await?))
}

/// Updates the budgeter's name and payee_ids.
#[tracing::instrument(skip_all)]
pub async fn update_budgeter<BS: BudgeterServiceExt>(
    Path(_id): Path<Uuid>,
    State(budgeter_service): State<BS>,
    WithRejection(Json(body), _): WithRejection<Json<BudgeterConfig>, JsonError>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(Json(budgeter_service.update_budgeter(body).await?))
}

/// Deletes the budgeter and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_budgeter<BS: BudgeterServiceExt>(
    Path(id): Path<Uuid>,
    State(budgeter_service): State<BS>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(Json(budgeter_service.delete_budgeter(id).await?))
}
