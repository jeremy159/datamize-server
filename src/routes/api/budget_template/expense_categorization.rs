use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    models::budget_template::ExpenseCategorization,
    services::budget_template::ExpenseCategorizationServiceExt,
};

/// Returns an expense categorization.
#[tracing::instrument(skip_all)]
pub async fn get_expense_categorization<ECS: ExpenseCategorizationServiceExt>(
    Path(id): Path<Uuid>,
    State(expense_categorization_service): State<ECS>,
) -> HttpJsonDatamizeResult<ExpenseCategorization> {
    Ok(Json(
        expense_categorization_service
            .get_expense_categorization(id)
            .await?,
    ))
}

/// Updates the expense categorization and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn update_expense_categorization<ECS: ExpenseCategorizationServiceExt>(
    Path(_id): Path<Uuid>,
    State(expense_categorization_service): State<ECS>,
    WithRejection(Json(body), _): WithRejection<Json<ExpenseCategorization>, JsonError>,
) -> HttpJsonDatamizeResult<ExpenseCategorization> {
    Ok(Json(
        expense_categorization_service
            .update_expense_categorization(body)
            .await?,
    ))
}
