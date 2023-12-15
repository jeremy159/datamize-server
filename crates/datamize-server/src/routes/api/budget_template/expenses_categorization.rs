use axum::{extract::State, Json};
use axum_extra::extract::WithRejection;
use datamize_domain::ExpenseCategorization;

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    services::budget_template::DynExpenseCategorizationService,
};

/// Returns all expenses categorization.
#[tracing::instrument(skip_all)]
pub async fn get_all_expenses_categorization(
    State(expense_categorization_service): State<DynExpenseCategorizationService>,
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
    State(expense_categorization_service): State<DynExpenseCategorizationService>,
    WithRejection(Json(body), _): WithRejection<Json<Vec<ExpenseCategorization>>, JsonError>,
) -> HttpJsonDatamizeResult<Vec<ExpenseCategorization>> {
    Ok(Json(
        expense_categorization_service
            .update_all_expenses_categorization(body)
            .await?,
    ))
}
