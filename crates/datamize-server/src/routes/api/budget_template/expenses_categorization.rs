use axum::extract::State;
use datamize_domain::ExpenseCategorization;

use crate::{
    error::{AppJson, HttpJsonDatamizeResult},
    services::budget_template::DynExpenseCategorizationService,
};

/// Returns all expenses categorization.
#[tracing::instrument(skip_all)]
pub async fn get_all_expenses_categorization(
    State(expense_categorization_service): State<DynExpenseCategorizationService>,
) -> HttpJsonDatamizeResult<Vec<ExpenseCategorization>> {
    Ok(AppJson(
        expense_categorization_service
            .get_all_expenses_categorization()
            .await?,
    ))
}

/// Updates all expenses categorization and returns the collection.
#[tracing::instrument(skip_all)]
pub async fn update_all_expenses_categorization(
    State(expense_categorization_service): State<DynExpenseCategorizationService>,
    AppJson(body): AppJson<Vec<ExpenseCategorization>>,
) -> HttpJsonDatamizeResult<Vec<ExpenseCategorization>> {
    Ok(AppJson(
        expense_categorization_service
            .update_all_expenses_categorization(body)
            .await?,
    ))
}
