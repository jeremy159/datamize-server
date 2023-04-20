use anyhow::Context;
use axum::{extract::State, Json};
use chrono::{Datelike, Local};
use uuid::Uuid;

use crate::{
    common::{update_month_net_totals, update_year_net_totals},
    db,
    domain::{Month, MonthNum},
    error::{AppError, HttpJsonAppResult},
    get_redis_conn,
    startup::AppState,
    web_scraper,
};

/// Endpoint to refresh non-editable financial resources.
/// Only resources from the current month will be refreshed by this endpoint.
/// If current month does not exists, it will create it.
/// This endpoint basically calls the YNAB api for some resources and starts a web scrapper for others.
/// Will return an array of ids for Financial Resources updated.
#[tracing::instrument(skip_all)]
pub async fn refresh_balance_sheet_resources(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Uuid>> {
    let db_conn_pool = app_state.db_conn_pool;
    let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get redis connection from pool")?;
    let ynab_client = app_state.ynab_client.as_ref();
    let current_date = Local::now().date_naive();
    let current_year = current_date.year();
    // The only condition is that the year exists...
    db::get_year_data(&db_conn_pool, current_year)
        .await
        .map_err(AppError::from_sqlx)?;

    let current_month: MonthNum = current_date.month().try_into().unwrap();
    if let Err(sqlx::Error::RowNotFound) =
        db::get_month_data(&db_conn_pool, current_month, current_year).await
    {
        // If month doesn't exist, create it
        let month = Month::new(current_month, current_year);
        db::add_new_month(&db_conn_pool, &month, current_year).await?;
    }

    let mut resources = db::get_financial_resources_of_year(&db_conn_pool, current_year)
        .await
        .map_err(AppError::from_sqlx)?;

    let accounts = ynab_client.get_accounts().await?;
    let external_accounts =
        web_scraper::get_external_accounts(&db_conn_pool, &mut redis_conn).await?;

    let mut refreshed = vec![];

    for res in &mut resources {
        if let Some(ref account_ids) = res.base.ynab_account_ids {
            if !account_ids.is_empty() {
                let balance = accounts
                    .iter()
                    .filter(|a| account_ids.contains(&a.id))
                    .map(|a| a.balance.abs())
                    .sum::<i64>();

                match res.balance_per_month.get_mut(&current_month) {
                    Some(current_balance) => {
                        if *current_balance != balance {
                            *current_balance = balance;
                            refreshed.push(res.base.id);
                        }
                    }
                    None => {
                        res.balance_per_month.insert(current_month, balance);
                        refreshed.push(res.base.id);
                    }
                }
            }
        }

        if let Some(ref account_ids) = res.base.external_account_ids {
            if !account_ids.is_empty() {
                let balance = external_accounts
                    .iter()
                    .filter(|a| account_ids.contains(&a.id))
                    .map(|a| a.balance.abs())
                    .sum::<i64>();

                match res.balance_per_month.get_mut(&current_month) {
                    Some(current_balance) => {
                        if *current_balance != balance {
                            *current_balance = balance;
                            refreshed.push(res.base.id);
                        }
                    }
                    None => {
                        res.balance_per_month.insert(current_month, balance);
                        refreshed.push(res.base.id);
                    }
                }
            }
        }
    }

    if !refreshed.is_empty() {
        resources.retain(|r| refreshed.contains(&r.base.id));
        for r in resources {
            db::update_financial_resource(&db_conn_pool, &r).await?;
        }
        update_month_net_totals(&db_conn_pool, current_month, current_year).await?;
        update_year_net_totals(&db_conn_pool, current_year).await?;
    }

    Ok(Json(refreshed))
}
