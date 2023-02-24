use std::cmp::Ordering;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    common::{build_months, create_month},
    db,
    domain::{Month, SaveMonth},
    error::{AppError, HttpJsonAppResult, JsonError},
    startup::AppState,
};

/// Returns all the months within a year with balance sheets.
#[tracing::instrument(name = "Get all months from a year", skip_all)]
pub async fn balance_sheet_months(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Month>> {
    let db_conn_pool = app_state.db_conn_pool;

    let Some(year_data) = db::get_year_data(&db_conn_pool, year)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    Ok(Json(build_months(&db_conn_pool, year_data).await?))
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

    let None = db::get_month_data(&db_conn_pool, year_data.id, body.month as i16)
    .await? else {
        return Err(AppError::MonthAlreadyExist);
    };

    let month = create_month(&db_conn_pool, year_data, body.month).await?;

    Ok((StatusCode::CREATED, Json(month)))
}

/// Returns all months of all years.
pub async fn all_balance_sheet_months(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Month>> {
    let db_conn_pool = app_state.db_conn_pool;

    let years_data = db::get_all_years_data(&db_conn_pool).await?;
    let mut months: Vec<Month> = vec![];

    for yd in years_data {
        months.extend(build_months(&db_conn_pool, yd).await?.into_iter());
    }

    months.sort_by(|a, b| match a.year.cmp(&b.year) {
        Ordering::Equal => a.month.cmp(&b.month),
        other => other,
    });

    Ok(Json(months))
}
