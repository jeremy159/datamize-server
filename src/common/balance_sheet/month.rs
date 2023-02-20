use std::collections::HashMap;

use futures::try_join;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::{self, YearData},
    domain::{Month, MonthNum},
    error::AppError,
};

#[tracing::instrument(skip(db_conn_pool))]
pub async fn build_months(db_conn_pool: &PgPool, year_id: Uuid) -> Result<Vec<Month>, AppError> {
    let months_data = db::get_months_data(db_conn_pool, year_id).await?;

    let mut months = HashMap::<Uuid, Month>::with_capacity(months_data.len());

    for month_data in &months_data {
        let net_totals_query = db::get_month_net_totals_for(db_conn_pool, month_data.id);

        let financial_resources_query =
            db::get_financial_resources_for(db_conn_pool, month_data.id);

        let (net_totals, resources) = try_join!(net_totals_query, financial_resources_query)?;

        months
            .entry(month_data.id)
            .and_modify(|m| {
                m.net_totals.extend(net_totals.clone());
                m.resources.extend(resources.clone())
            })
            .or_insert_with(|| Month {
                id: month_data.id,
                month: MonthNum::try_from(month_data.month).unwrap(),
                net_totals,
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
    year_id: Uuid,
    month: MonthNum,
) -> Result<Month, AppError> {
    let Some(month_data) = db::get_month_data(db_conn_pool, year_id, month as i16)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let net_totals_query = db::get_month_net_totals_for(db_conn_pool, month_data.id);

    let financial_resources_query = db::get_financial_resources_for(db_conn_pool, month_data.id);

    let (net_totals, resources) = try_join!(net_totals_query, financial_resources_query)?;

    Ok(Month {
        id: month_data.id,
        month: month_data.month.try_into().unwrap(),
        net_totals,
        resources,
    })
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn create_month(
    db_conn_pool: &PgPool,
    year_data: YearData,
    month_num: MonthNum,
) -> Result<Month, AppError> {
    let mut month = Month::new(month_num);

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
                month.update_net_totals_with_previous(&prev_net_totals);
            }
        }
    }

    db::add_new_month(db_conn_pool, &month, year_data.id).await?;

    Ok(month)
}
