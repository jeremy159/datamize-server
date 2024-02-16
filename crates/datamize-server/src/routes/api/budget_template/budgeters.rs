use axum::{extract::State, http::StatusCode, response::IntoResponse};
use datamize_domain::{BudgeterConfig, SaveBudgeterConfig};

use crate::{
    error::{AppError, AppJson, HttpJsonDatamizeResult},
    services::budget_template::DynBudgeterService,
};

/// Returns all the budgeters.
#[tracing::instrument(skip_all)]
pub async fn get_all_budgeters(
    State(budgeter_service): State<DynBudgeterService>,
) -> HttpJsonDatamizeResult<Vec<BudgeterConfig>> {
    Ok(AppJson(budgeter_service.get_all_budgeters().await?))
}

/// Creates a new budgeter if it doesn't already exist and returns the newly created entity.
#[tracing::instrument(skip_all)]
pub async fn create_budgeter(
    State(budgeter_service): State<DynBudgeterService>,
    AppJson(body): AppJson<SaveBudgeterConfig>,
) -> impl IntoResponse {
    Ok::<_, AppError>((
        StatusCode::CREATED,
        AppJson(budgeter_service.create_budgeter(body).await?),
    ))
}
