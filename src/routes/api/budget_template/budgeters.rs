use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;

use crate::{
    db::budget_template::{
        get_all_budgeters_config, get_budgeter_config_by_name, update_budgeter_config,
    },
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    models::budget_template::{BudgeterConfig, SaveBudgeterConfig},
    startup::AppState,
};

/// Returns all the budgeters.
#[tracing::instrument(skip_all)]
pub async fn get_all_budgeters(
    State(app_state): State<AppState>,
) -> HttpJsonDatamizeResult<Vec<BudgeterConfig>> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(get_all_budgeters_config(&db_conn_pool).await?))
}

/// Creates a new budgeter if it doesn't already exist and returns the newly created entity.
#[tracing::instrument(skip_all)]
pub async fn create_budgeter(
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<SaveBudgeterConfig>, JsonError>,
) -> impl IntoResponse {
    let db_conn_pool = app_state.db_conn_pool;

    let Err(sqlx::Error::RowNotFound) =
        get_budgeter_config_by_name(&db_conn_pool, &body.name).await else
    {
        return Err(AppError::ResourceAlreadyExist);
    };

    let budgeter_config: BudgeterConfig = body.into();
    update_budgeter_config(&db_conn_pool, &budgeter_config).await?;

    Ok((StatusCode::CREATED, Json(budgeter_config)))
}
