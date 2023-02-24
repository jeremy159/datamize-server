use std::collections::HashMap;

use futures::try_join;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::{self, YearData},
    domain::{Month, MonthNum, NetTotal, NetTotalType},
    error::AppError,
};

#[tracing::instrument(skip(db_conn_pool))]
pub async fn build_months(
    db_conn_pool: &PgPool,
    year_data: YearData,
) -> Result<Vec<Month>, AppError> {
    let months_data = db::get_months_data(db_conn_pool, year_data.id).await?;

    let mut months = HashMap::<Uuid, Month>::with_capacity(months_data.len());

    for month_data in &months_data {
        let net_totals_query = db::get_month_net_totals_for(db_conn_pool, month_data.id);

        let financial_resources_query =
            db::get_financial_resources_for(db_conn_pool, month_data.id);

        let (net_totals, resources) = try_join!(net_totals_query, financial_resources_query)?;

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

        months
            .entry(month_data.id)
            .and_modify(|m| {
                m.net_assets = net_assets.clone();
                m.net_portfolio = net_portfolio.clone();
                m.resources.extend(resources.clone())
            })
            .or_insert_with(|| Month {
                id: month_data.id,
                month: MonthNum::try_from(month_data.month).unwrap(),
                year: year_data.year,
                net_assets,
                net_portfolio,
                resources,
            });
    }

    let mut months = months.into_values().collect::<Vec<_>>();

    months.sort_by(|a, b| a.month.cmp(&b.month));

    Ok(months)
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_month(
    db_conn_pool: &PgPool,
    year_data: YearData,
    month: MonthNum,
) -> Result<Month, AppError> {
    let Some(month_data) = db::get_month_data(db_conn_pool, year_data.id, month as i16)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let net_totals_query = db::get_month_net_totals_for(db_conn_pool, month_data.id);

    let financial_resources_query = db::get_financial_resources_for(db_conn_pool, month_data.id);

    let (net_totals, resources) = try_join!(net_totals_query, financial_resources_query)?;
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

    Ok(Month {
        id: month_data.id,
        month: month_data.month.try_into().unwrap(),
        year: year_data.year,
        net_assets,
        net_portfolio,
        resources,
    })
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn create_month(
    db_conn_pool: &PgPool,
    year_data: YearData,
    month_num: MonthNum,
) -> Result<Month, AppError> {
    let mut month = Month::new(month_num, year_data.year);

    let year_data_opt = match month_num.pred() {
        MonthNum::December => db::get_year_data(db_conn_pool, year_data.year - 1).await,
        _ => Ok(Some(year_data)),
    };

    if let Ok(Some(year_data)) = year_data_opt {
        if let Ok(Some(prev_month)) =
            db::get_month_data(db_conn_pool, year_data.id, month_num.pred() as i16).await
        {
            if let Ok(prev_net_totals) =
                db::get_month_net_totals_for(db_conn_pool, prev_month.id).await
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

    db::add_new_month(db_conn_pool, &month, year_data.id).await?;

    Ok(month)
}
