use async_recursion::async_recursion;
use sqlx::PgPool;

use crate::{
    db::balance_sheet::{
        get_month, get_month_data, get_month_net_totals_for, get_year_data,
        insert_monthly_net_totals,
    },
    error::AppError,
    models::balance_sheet::{Month, MonthNum, NetTotalType},
};

#[tracing::instrument(skip(db_conn_pool))]
#[async_recursion]
pub async fn update_month_net_totals(
    db_conn_pool: &PgPool,
    month_num: MonthNum,
    year: i32,
) -> Result<Month, AppError> {
    get_month_data(db_conn_pool, month_num, year)
        .await
        .map_err(AppError::from_sqlx)?;

    let mut month = get_month(db_conn_pool, month_num, year).await?;

    month.compute_net_totals();

    let prev_year = match month_num.pred() {
        MonthNum::December => year - 1,
        _ => year,
    };

    if let Ok(prev_month) = get_month_data(db_conn_pool, month_num.pred(), prev_year).await {
        if let Ok(prev_net_totals) = get_month_net_totals_for(db_conn_pool, prev_month.id).await {
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

    insert_monthly_net_totals(
        db_conn_pool,
        month.id,
        [&month.net_assets, &month.net_portfolio],
    )
    .await?;

    let next_year_num = match month_num.succ() {
        MonthNum::January => year + 1,
        _ => year,
    };

    // Should also try to update next month if it exists
    if (get_year_data(db_conn_pool, next_year_num).await).is_ok() {
        if let Ok(next_month) = get_month_data(db_conn_pool, month_num.succ(), next_year_num).await
        {
            update_month_net_totals(
                db_conn_pool,
                next_month.month.try_into().unwrap(),
                next_year_num,
            )
            .await?;
        }
    }

    Ok(month)
}
