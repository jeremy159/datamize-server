use async_recursion::async_recursion;
use futures::try_join;
use sqlx::PgPool;

use crate::{
    db,
    domain::{NetTotal, NetTotalType, YearDetail},
    error::AppError,
};

#[tracing::instrument(skip_all)]
pub async fn get_year(db_conn_pool: &PgPool, year: i32) -> Result<YearDetail, AppError> {
    let year_data = db::get_year_data(db_conn_pool, year)
        .await
        .map_err(AppError::from_sqlx)?;

    let (net_totals, saving_rates, months, resources) = try_join!(
        db::get_year_net_totals_for(db_conn_pool, year_data.id),
        db::get_saving_rates_for(db_conn_pool, year_data.id),
        db::get_months(db_conn_pool, year),
        db::get_financial_resources_of_year(db_conn_pool, year_data.year),
    )?;

    let net_assets = match net_totals
        .clone()
        .into_iter()
        .find(|nt| nt.net_type == NetTotalType::Asset)
    {
        Some(na) => na,
        None => NetTotal::new_asset(),
    };
    let net_portfolio = match net_totals
        .into_iter()
        .find(|nt| nt.net_type == NetTotalType::Portfolio)
    {
        Some(np) => np,
        None => NetTotal::new_portfolio(),
    };

    let year = YearDetail {
        id: year_data.id,
        year: year_data.year,
        refreshed_at: year_data.refreshed_at,
        net_assets,
        net_portfolio,
        saving_rates,
        resources,
        months,
    };

    Ok(year)
}

#[async_recursion]
pub async fn update_year_net_totals(
    db_conn_pool: &PgPool,
    year: i32,
) -> Result<YearDetail, AppError> {
    let year_data = db::get_year_data(db_conn_pool, year)
        .await
        .map_err(AppError::from_sqlx)?;

    let (net_totals, saving_rates, months) = try_join!(
        db::get_year_net_totals_for(db_conn_pool, year_data.id),
        db::get_saving_rates_for(db_conn_pool, year_data.id),
        db::get_months(db_conn_pool, year),
    )?;

    let net_assets = match net_totals
        .clone()
        .into_iter()
        .find(|nt| nt.net_type == NetTotalType::Asset)
    {
        Some(na) => na,
        None => NetTotal::new_asset(),
    };
    let net_portfolio = match net_totals
        .into_iter()
        .find(|nt| nt.net_type == NetTotalType::Portfolio)
    {
        Some(np) => np,
        None => NetTotal::new_portfolio(),
    };

    let mut year = YearDetail {
        id: year_data.id,
        year: year_data.year,
        refreshed_at: year_data.refreshed_at,
        net_assets,
        net_portfolio,
        saving_rates,
        resources: vec![],
        months,
    };

    if let Some(last_month) = year.get_last_month() {
        if year.needs_net_totals_update(&last_month.net_assets, &last_month.net_portfolio) {
            year.update_net_assets_with_last_month(&last_month.net_assets);
            year.update_net_portfolio_with_last_month(&last_month.net_portfolio);
        }
    }

    // Also update with previous year since we might just have updated the total balance of current year.
    if let Ok(prev_year) = db::get_year_data(db_conn_pool, year.year - 1).await {
        if let Ok(prev_net_totals) = db::get_year_net_totals_for(db_conn_pool, prev_year.id).await {
            if let Some(prev_net_assets) = prev_net_totals
                .iter()
                .find(|pnt| pnt.net_type == NetTotalType::Asset)
            {
                year.update_net_assets_with_previous(prev_net_assets);
            }
            if let Some(prev_net_portfolio) = prev_net_totals
                .iter()
                .find(|pnt| pnt.net_type == NetTotalType::Portfolio)
            {
                year.update_net_portfolio_with_previous(prev_net_portfolio);
            }
        }
    }

    // Should also try to update next year if it exists
    if let Ok(next_year) = db::get_year_data(db_conn_pool, year.year + 1).await {
        update_year_net_totals(db_conn_pool, next_year.year).await?;
    }

    db::insert_yearly_net_totals(
        db_conn_pool,
        year.id,
        [&year.net_assets, &year.net_portfolio],
    )
    .await?;

    Ok(year)
}
