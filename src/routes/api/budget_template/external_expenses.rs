use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;

use crate::{
    error::{DatamizeResult, HttpJsonDatamizeResult, JsonError},
    models::budget_template::{ExternalExpense, SaveExternalExpense},
    services::budget_template::ExternalExpenseServiceExt,
};

/// Returns all external_expenses.
#[tracing::instrument(skip_all)]
pub async fn get_all_external_expenses<EES: ExternalExpenseServiceExt>(
    State(external_expense_service): State<EES>,
) -> HttpJsonDatamizeResult<Vec<ExternalExpense>> {
    Ok(Json(
        external_expense_service.get_all_external_expenses().await?,
    ))
}

/// Creates a new budgeter if it doesn't already exist and returns the newly created entity.
#[tracing::instrument(skip_all)]
pub async fn create_external_expense<EES: ExternalExpenseServiceExt>(
    State(external_expense_service): State<EES>,
    WithRejection(Json(body), _): WithRejection<Json<SaveExternalExpense>, JsonError>,
) -> DatamizeResult<impl IntoResponse> {
    Ok((
        StatusCode::CREATED,
        Json(
            external_expense_service
                .create_external_expense(body)
                .await?,
        ),
    ))
}
