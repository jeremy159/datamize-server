use std::collections::HashMap;

use futures::try_join;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db,
    domain::{Month, MonthNum},
    error::AppError,
};

#[tracing::instrument(skip_all)]
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

#[tracing::instrument(skip_all)]
pub async fn get_month(
    db_conn_pool: &PgPool,
    year_id: Uuid,
    month: MonthNum,
) -> Result<Month, AppError> {
    let Some(month_data) = db::get_month_data(db_conn_pool, year_id, month as i16)
    .await? else {
        return Err(crate::error::AppError::ResourceNotFound);
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
