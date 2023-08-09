use axum::{extract::State, Json};
use axum_extra::extract::WithRejection;

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    models::budget_template::ExpenseCategorization,
    services::budget_template::{ExpenseCategorizationService, ExpenseCategorizationServiceExt},
};

/// Returns all expenses categorization.
#[tracing::instrument(skip_all)]
pub async fn get_all_expenses_categorization(
    State(expense_categorization_service): State<ExpenseCategorizationService>,
) -> HttpJsonDatamizeResult<Vec<ExpenseCategorization>> {
    Ok(Json(
        expense_categorization_service
            .get_all_expenses_categorization()
            .await?,
    ))
}

/// Updates all expenses categorization and returns the collection.
#[tracing::instrument(skip_all)]
pub async fn update_all_expenses_categorization(
    State(expense_categorization_service): State<ExpenseCategorizationService>,
    WithRejection(Json(body), _): WithRejection<Json<Vec<ExpenseCategorization>>, JsonError>,
) -> HttpJsonDatamizeResult<Vec<ExpenseCategorization>> {
    Ok(Json(
        expense_categorization_service
            .update_all_expenses_categorization(body)
            .await?,
    ))
}
