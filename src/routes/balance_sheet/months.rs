use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    common::build_months,
    db,
    domain::{Month, SaveMonth},
    error::{AppError, HttpJsonAppResult},
    startup::AppState,
};

pub async fn balance_sheet_months(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Month>> {
    let db_conn_pool = app_state.db_conn_pool;

    let Some(year_data) = db::get_year_data(&db_conn_pool, year)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    Ok(Json(build_months(&db_conn_pool, year_data.id).await?))
}

// TODO: When creating month, update net totals for this month compared to previous one if any.
pub async fn create_balance_sheet_month(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
    Json(body): Json<SaveMonth>,
) -> HttpJsonAppResult<Month> {
    let db_conn_pool = app_state.db_conn_pool;

    let Some(year_data) = db::get_year_data(&db_conn_pool, year)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let None = db::get_month_data(&db_conn_pool, year_data.id, body.month as i16)
    .await? else {
        return Err(AppError::MonthAlreadyExist);
    };

    let month = Month::new(body.month);

    db::add_new_month(&db_conn_pool, &month, year_data.id).await?;

    Ok(Json(month))
}
