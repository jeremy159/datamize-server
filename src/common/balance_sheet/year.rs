use futures::try_join;
use sqlx::PgPool;

use crate::{db, domain::YearDetail, error::AppError};

use super::build_months;

pub async fn get_year(db_conn_pool: &PgPool, year: i32) -> Result<YearDetail, AppError> {
    let Some(year_data) = db::get_year_data(db_conn_pool, year)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let net_totals_query = db::get_year_net_totals_for(db_conn_pool, year_data.id);

    let saving_rates_query = db::get_saving_rates_for(db_conn_pool, year_data.id);

    let (net_totals, saving_rates) = try_join!(net_totals_query, saving_rates_query)?;
    let months = build_months(db_conn_pool, year_data.id).await?;

    Ok(YearDetail {
        id: year_data.id,
        year: year_data.year,
        net_totals,
        saving_rates,
        months,
    })
}
