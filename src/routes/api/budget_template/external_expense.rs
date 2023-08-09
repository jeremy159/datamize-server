use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    models::budget_template::ExternalExpense,
    services::budget_template::{ExternalExpenseService, ExternalExpenseServiceExt},
};

/// Returns an external expense.
#[tracing::instrument(skip_all)]
pub async fn get_external_expense(
    Path(id): Path<Uuid>,
    State(external_expense_service): State<ExternalExpenseService>,
) -> HttpJsonDatamizeResult<ExternalExpense> {
    Ok(Json(
        external_expense_service.get_external_expense(id).await?,
    ))
}

/// Updates the external expense and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn update_external_expense(
    Path(_id): Path<Uuid>,
    State(external_expense_service): State<ExternalExpenseService>,
    WithRejection(Json(body), _): WithRejection<Json<ExternalExpense>, JsonError>,
) -> HttpJsonDatamizeResult<ExternalExpense> {
    Ok(Json(
        external_expense_service
            .update_external_expense(body)
            .await?,
    ))
}

/// Deletes the external expense and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_external_expense(
    Path(id): Path<Uuid>,
    State(external_expense_service): State<ExternalExpenseService>,
) -> HttpJsonDatamizeResult<ExternalExpense> {
    Ok(Json(
        external_expense_service.delete_external_expense(id).await?,
    ))
}
