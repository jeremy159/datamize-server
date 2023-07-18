use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use crate::{
    db::budget_template as db,
    error::{AppError, HttpJsonAppResult, JsonError},
    models::budget_template::ExternalExpense,
    startup::AppState,
};

/// Returns an external expense.
#[tracing::instrument(skip_all)]
pub async fn get_external_expense(
    Path(id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<ExternalExpense> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(db::get_external_expense(&db_conn_pool, id).await?))
}

/// Updates the external expense and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn update_external_expense(
    Path(_id): Path<Uuid>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<ExternalExpense>, JsonError>,
) -> HttpJsonAppResult<ExternalExpense> {
    let db_conn_pool = app_state.db_conn_pool;

    let Ok(_) = db::get_external_expense(&db_conn_pool, body.id).await else {
        return Err(AppError::ResourceNotFound);
    };

    db::update_external_expense(&db_conn_pool, &body).await?;

    Ok(Json(body))
}

/// Deletes the external expense and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_external_expense(
    Path(id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<ExternalExpense> {
    let db_conn_pool = app_state.db_conn_pool;

    let Ok(external_expense) = db::get_external_expense(&db_conn_pool, id).await else {
        return Err(AppError::ResourceNotFound);
    };
    db::delete_external_expense(&db_conn_pool, id).await?;

    Ok(Json(external_expense))
}
