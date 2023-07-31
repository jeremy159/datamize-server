use axum::{extract::State, Json};
use axum_extra::extract::WithRejection;

use crate::{
    db::budget_template as db,
    error::{HttpJsonDatamizeResult, JsonError},
    models::budget_template::ExpenseCategorization,
    startup::AppState,
};

/// Returns all expenses categorization.
#[tracing::instrument(skip_all)]
pub async fn get_all_expenses_categorization(
    State(app_state): State<AppState>,
) -> HttpJsonDatamizeResult<Vec<ExpenseCategorization>> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(
        db::get_all_expenses_categorization(&db_conn_pool).await?,
    ))
}

/// Updates all expenses categorization and returns the collection.
#[tracing::instrument(skip_all)]
pub async fn update_all_expenses_categorization(
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<Vec<ExpenseCategorization>>, JsonError>,
) -> HttpJsonDatamizeResult<Vec<ExpenseCategorization>> {
    let db_conn_pool = app_state.db_conn_pool;

    let expenses_categorization = body;
    db::update_all_expenses_categorization(&db_conn_pool, &expenses_categorization).await?;

    Ok(Json(expenses_categorization))
}
