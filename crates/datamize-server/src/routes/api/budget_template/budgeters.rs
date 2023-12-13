use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;
use datamize_domain::{BudgeterConfig, SaveBudgeterConfig};

use crate::{
    error::{DatamizeResult, HttpJsonDatamizeResult, JsonError},
    services::budget_template::DynBudgeterService,
};

/// Returns all the budgeters.
#[tracing::instrument(skip_all)]
pub async fn get_all_budgeters(
    State(budgeter_service): State<DynBudgeterService>,
) -> HttpJsonDatamizeResult<Vec<BudgeterConfig>> {
    Ok(Json(budgeter_service.get_all_budgeters().await?))
}

/// Creates a new budgeter if it doesn't already exist and returns the newly created entity.
#[tracing::instrument(skip_all)]
pub async fn create_budgeter(
    State(budgeter_service): State<DynBudgeterService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveBudgeterConfig>, JsonError>,
) -> DatamizeResult<impl IntoResponse> {
    Ok((
        StatusCode::CREATED,
        Json(budgeter_service.create_budgeter(body).await?),
    ))
}
