use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;

use crate::{
    db::budget_template as db,
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    models::budget_template::{ExternalExpense, SaveExternalExpense},
    startup::AppState,
};

/// Returns all external_expenses.
#[tracing::instrument(skip_all)]
pub async fn get_all_external_expenses(
    State(app_state): State<AppState>,
) -> HttpJsonDatamizeResult<Vec<ExternalExpense>> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(db::get_all_external_expenses(&db_conn_pool).await?))
}

/// Creates a new budgeter if it doesn't already exist and returns the newly created entity.
#[tracing::instrument(skip_all)]
pub async fn create_external_expense(
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<SaveExternalExpense>, JsonError>,
) -> impl IntoResponse {
    let db_conn_pool = app_state.db_conn_pool;

    let Err(sqlx::Error::RowNotFound) =
        db::get_external_expense_by_name(&db_conn_pool, &body.name).await else
    {
        return Err(AppError::ResourceAlreadyExist);
    };

    let external_expense: ExternalExpense = body.into();
    db::update_external_expense(&db_conn_pool, &external_expense).await?;

    Ok((StatusCode::CREATED, Json(external_expense)))
}
