use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    common::update_month_net_totals,
    db,
    domain::{Month, SaveMonth},
    error::{AppError, HttpJsonAppResult, JsonError},
    startup::AppState,
};

/// Returns all months of all years.
#[tracing::instrument(name = "Get all months from all years", skip_all)]
pub async fn all_balance_sheet_months(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Month>> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(db::get_all_months(&db_conn_pool).await?))
}

/// Returns all the months within a year with balance sheets.
#[tracing::instrument(name = "Get all months from a year", skip_all)]
pub async fn balance_sheet_months(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Month>> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(db::get_months(&db_conn_pool, year).await?))
}

/// Creates a new month if it doesn't already exist and returns the newly created entity.
/// Will also update net totals for this month compared to previous one if any.
#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_month(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<SaveMonth>, JsonError>,
) -> impl IntoResponse {
    let db_conn_pool = app_state.db_conn_pool;

    let Some(year_data) = db::get_year_data(&db_conn_pool, year)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let None = db::get_month_data(&db_conn_pool, body.month, year)
    .await? else {
        return Err(AppError::MonthAlreadyExist);
    };

    let month = Month::new(body.month, year);
    db::add_new_month(&db_conn_pool, &month, year_data.id).await?;

    let month = update_month_net_totals(&db_conn_pool, body.month, year).await?;

    Ok((StatusCode::CREATED, Json(month)))
}
