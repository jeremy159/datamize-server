use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    common::build_months,
    db,
    domain::{Month, MonthNum, SaveMonth},
    error::{AppError, HttpJsonAppResult},
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

    Ok(Json(build_months(&db_conn_pool, year_data.id).await?))
}

/// Creates a new month if it doesn't already exist and returns the newly created entity.
/// Will also update net totals for this month compared to previous one if any.
#[tracing::instrument(skip_all)]
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

    let mut month = Month::new(body.month);

    let year_data_opt = match body.month.pred() {
        MonthNum::December => db::get_year_data(&db_conn_pool, year - 1).await,
        _ => Ok(Some(year_data)),
    };

    if let Ok(Some(year_data)) = year_data_opt {
        if let Ok(Some(prev_month)) =
            db::get_month_data(&db_conn_pool, year_data.id, body.month.pred() as i16).await
        {
            if let Ok(prev_net_totals) =
                db::get_month_net_totals_for(&db_conn_pool, prev_month.id).await
            {
                month.update_net_totals_with_previous(&prev_net_totals);
            }
        }
    }

    db::add_new_month(&db_conn_pool, &month, year_data.id).await?;

    Ok(Json(month))
}
