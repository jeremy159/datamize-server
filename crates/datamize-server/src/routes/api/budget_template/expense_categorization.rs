use axum::extract::{Path, State};
use datamize_domain::{ExpenseCategorization, Uuid};

use crate::{
    error::{AppJson, HttpJsonDatamizeResult},
    services::budget_template::DynExpenseCategorizationService,
};

/// Returns an expense categorization.
#[tracing::instrument(skip_all)]
pub async fn get_expense_categorization(
    Path(id): Path<Uuid>,
    State(expense_categorization_service): State<DynExpenseCategorizationService>,
) -> HttpJsonDatamizeResult<ExpenseCategorization> {
    Ok(AppJson(
        expense_categorization_service
            .get_expense_categorization(id)
            .await?,
    ))
}

/// Updates the expense categorization and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn update_expense_categorization(
    Path(_id): Path<Uuid>,
    State(expense_categorization_service): State<DynExpenseCategorizationService>,
    AppJson(body): AppJson<ExpenseCategorization>,
) -> HttpJsonDatamizeResult<ExpenseCategorization> {
    Ok(AppJson(
        expense_categorization_service
            .update_expense_categorization(body)
            .await?,
    ))
}
