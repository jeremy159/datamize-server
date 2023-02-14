use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    common::get_year,
    db,
    domain::{UpdateYear, YearDetail},
    error::HttpJsonAppResult,
    startup::AppState,
};

/// Returns a detailed year with its balance sheet and its saving rates.
pub async fn balance_sheet_year(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<YearDetail> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(get_year(&db_conn_pool, year).await?))
}

/// Updates the saving rates of the received year.
pub async fn update_balance_sheet_year(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
    Json(body): Json<UpdateYear>,
) -> HttpJsonAppResult<YearDetail> {
    let db_conn_pool = app_state.db_conn_pool;

    let mut year = get_year(&db_conn_pool, year).await?;

    db::update_saving_rates(&db_conn_pool, &body.saving_rates).await?;

    year.update_saving_rates(body.saving_rates);

    Ok(Json(year))
}
