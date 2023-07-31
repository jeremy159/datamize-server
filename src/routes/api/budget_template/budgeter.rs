use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use crate::{
    db::budget_template::{delete_budgeter_config, get_budgeter_config, update_budgeter_config},
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    models::budget_template::BudgeterConfig,
    startup::AppState,
};

/// Returns a budgeter's config.
#[tracing::instrument(skip_all)]
pub async fn get_budgeter(
    Path(id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(get_budgeter_config(&db_conn_pool, id).await?))
}

/// Updates the budgeter's name and payee_ids.
#[tracing::instrument(skip_all)]
pub async fn update_budgeter(
    Path(_id): Path<Uuid>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<BudgeterConfig>, JsonError>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    let db_conn_pool = app_state.db_conn_pool;

    let Ok(_) = get_budgeter_config(&db_conn_pool, body.id).await else {
        return Err(AppError::ResourceNotFound);
    };

    update_budgeter_config(&db_conn_pool, &body).await?;

    Ok(Json(body))
}

/// Deletes the budgeter and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_budgeter(
    Path(id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    let db_conn_pool = app_state.db_conn_pool;

    let Ok(budgeter_config) = get_budgeter_config(&db_conn_pool, id).await else {
        return Err(AppError::ResourceNotFound);
    };
    delete_budgeter_config(&db_conn_pool, id).await?;

    Ok(Json(budgeter_config))
}
