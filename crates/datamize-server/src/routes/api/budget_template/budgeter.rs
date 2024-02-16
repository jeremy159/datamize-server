use axum::extract::{Path, State};
use datamize_domain::{BudgeterConfig, Uuid};

use crate::{
    error::{AppJson, HttpJsonDatamizeResult},
    services::budget_template::DynBudgeterService,
};

/// Returns a budgeter's config.
#[tracing::instrument(skip_all)]
pub async fn get_budgeter(
    Path(id): Path<Uuid>,
    State(budgeter_service): State<DynBudgeterService>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(AppJson(budgeter_service.get_budgeter(id).await?))
}

/// Updates the budgeter's name and payee_ids.
#[tracing::instrument(skip_all)]
pub async fn update_budgeter(
    Path(_id): Path<Uuid>,
    State(budgeter_service): State<DynBudgeterService>,
    AppJson(body): AppJson<BudgeterConfig>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(AppJson(budgeter_service.update_budgeter(body).await?))
}

/// Deletes the budgeter and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_budgeter(
    Path(id): Path<Uuid>,
    State(budgeter_service): State<DynBudgeterService>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(AppJson(budgeter_service.delete_budgeter(id).await?))
}
