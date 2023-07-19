use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use crate::{
    db::budget_template as db,
    error::{AppError, HttpJsonAppResult, JsonError},
    models::budget_template::ExpenseCategorization,
    startup::AppState,
};

/// Returns an expense categorization.
#[tracing::instrument(skip_all)]
pub async fn get_expense_categorization(
    Path(id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<ExpenseCategorization> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(
        db::get_expense_categorization(&db_conn_pool, id).await?,
    ))
}

/// Updates the expense categorization and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn update_expense_categorization(
    Path(_id): Path<Uuid>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<ExpenseCategorization>, JsonError>,
) -> HttpJsonAppResult<ExpenseCategorization> {
    let db_conn_pool = app_state.db_conn_pool;

    let Ok(_) = db::get_expense_categorization(&db_conn_pool, body.id).await else {
        return Err(AppError::ResourceNotFound);
    };

    db::update_expense_categorization(&db_conn_pool, &body).await?;

    Ok(Json(body))
}
