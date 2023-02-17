use futures::{try_join, TryFutureExt};
use sqlx::PgPool;

use crate::{db, domain::YearDetail, error::AppError};

use super::build_months;

#[tracing::instrument(skip_all)]
pub async fn get_year(db_conn_pool: &PgPool, year: i32) -> Result<YearDetail, AppError> {
    let Some(year_data) = db::get_year_data(db_conn_pool, year)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let net_totals_query = db::get_year_net_totals_for(db_conn_pool, year_data.id);

    let saving_rates_query = db::get_saving_rates_for(db_conn_pool, year_data.id);

    let (net_totals, saving_rates, months) = try_join!(
        net_totals_query.map_err(AppError::from),
        saving_rates_query.map_err(AppError::from),
        build_months(db_conn_pool, year_data.id).map_err(AppError::from)
    )?;

    let mut year = YearDetail {
        id: year_data.id,
        year: year_data.year,
        net_totals,
        saving_rates,
        months,
    };

    if let Some(last_month) = year.get_last_month() {
        if year.needs_net_totals_update(&last_month.net_totals) {
            year.update_net_totals_with_last_month(&last_month.net_totals);

            // Also update with previous year since we just updated the total balance of current year.
            if let Ok(Some(prev_year)) = db::get_year_data(db_conn_pool, year.year - 1).await {
                if let Ok(prev_net_totals) =
                    db::get_year_net_totals_for(db_conn_pool, prev_year.id).await
                {
                    year.update_net_totals_with_previous(&prev_net_totals);
                }
            }

            db::update_year_net_totals(db_conn_pool, &year).await?;
        }
    }

    Ok(year)
}
