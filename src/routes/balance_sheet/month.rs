use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    common::get_month,
    db,
    domain::{Month, MonthNum, UpdateMonth},
    error::{AppError, HttpJsonAppResult},
    startup::AppState,
};

/// Returns a specific month with its financial resources and net totals.
pub async fn balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Month> {
    let db_conn_pool = app_state.db_conn_pool;

    let Some(year_data) = db::get_year_data(&db_conn_pool, year)
    .await? else {
        return Err(crate::error::AppError::ResourceNotFound);
    };

    Ok(Json(get_month(&db_conn_pool, year_data.id, month).await?))
}

/// Updates the month, i.e. all the financial resources included in the month
/// Will also update its net totals.
pub async fn update_balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(app_state): State<AppState>,
    Json(body): Json<UpdateMonth>,
) -> HttpJsonAppResult<Month> {
    let db_conn_pool = app_state.db_conn_pool;

    let Some(year_data) = db::get_year_data(&db_conn_pool, year)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let mut month = get_month(&db_conn_pool, year_data.id, month).await?;

    db::update_financial_resources(&db_conn_pool, &body.resources).await?;

    month.update_financial_resources(body.resources);
    month.compute_net_totals();

    let year_data_opt = match month.month.pred() {
        MonthNum::December => db::get_year_data(&db_conn_pool, year - 1).await,
        _ => Ok(Some(year_data)),
    };

    if let Ok(Some(year_data)) = year_data_opt {
        if let Ok(Some(prev_month)) =
            db::get_month_data(&db_conn_pool, year_data.id, month.month.pred() as i16).await
        {
            if let Ok(prev_net_totals) =
                db::get_month_net_totals_for(&db_conn_pool, prev_month.id).await
            {
                month.update_net_totals_with_previous(&prev_net_totals);
            }
        }
    }

    db::update_month_net_totals(&db_conn_pool, &month.net_totals).await?;

    Ok(Json(month))
}
