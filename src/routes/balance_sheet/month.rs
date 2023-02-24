use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    common::get_month,
    db,
    domain::{Month, MonthNum, NetTotalType, UpdateMonth},
    error::{AppError, HttpJsonAppResult, JsonError},
    startup::AppState,
};

/// Returns a specific month with its financial resources and net totals.
#[tracing::instrument(name = "Get a month", skip_all)]
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
#[tracing::instrument(skip_all)]
pub async fn update_balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<UpdateMonth>, JsonError>,
) -> HttpJsonAppResult<Month> {
    let db_conn_pool = app_state.db_conn_pool;

    let Some(year_data) = db::get_year_data(&db_conn_pool, year)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let mut month = get_month(&db_conn_pool, year_data.id, month).await?;
    month.update_financial_resources(body.resources);

    db::update_financial_resources(&db_conn_pool, &month).await?;

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
                if let Some(prev_net_assets) = prev_net_totals
                    .iter()
                    .find(|pnt| pnt.net_type == NetTotalType::Asset)
                {
                    month.update_net_assets_with_previous(prev_net_assets);
                }
                if let Some(prev_net_portfolio) = prev_net_totals
                    .iter()
                    .find(|pnt| pnt.net_type == NetTotalType::Portfolio)
                {
                    month.update_net_portfolio_with_previous(prev_net_portfolio);
                }
            }
        }
    }

    db::insert_monthly_net_totals(
        &db_conn_pool,
        month.id,
        [&month.net_assets, &month.net_portfolio],
    )
    .await?;

    Ok(Json(month))
}
