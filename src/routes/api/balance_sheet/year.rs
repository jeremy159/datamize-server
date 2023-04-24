use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    common::get_year,
    db,
    domain::{UpdateYear, YearDetail},
    error::{HttpJsonAppResult, JsonError},
    startup::AppState,
};

/// Returns a detailed year with its balance sheet, its saving rates, its months and its financial resources.
#[tracing::instrument(name = "Get a detailed year", skip_all)]
pub async fn balance_sheet_year(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<YearDetail> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(get_year(&db_conn_pool, year).await?))
}

/// Updates the saving rates of the received year.
#[tracing::instrument(skip_all)]
pub async fn update_balance_sheet_year(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<UpdateYear>, JsonError>,
) -> HttpJsonAppResult<YearDetail> {
    let db_conn_pool = app_state.db_conn_pool;

    let mut year = get_year(&db_conn_pool, year).await?;
    year.update_saving_rates(body.saving_rates);

    db::update_saving_rates(&db_conn_pool, &year).await?;

    Ok(Json(year))
}

/// Deletes the year and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_year(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<YearDetail> {
    let db_conn_pool = app_state.db_conn_pool;

    let year_detail = get_year(&db_conn_pool, year).await?;
    db::delete_year(&db_conn_pool, year).await?;

    Ok(Json(year_detail))
}
