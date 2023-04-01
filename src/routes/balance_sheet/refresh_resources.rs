use std::collections::HashMap;

use axum::{extract::State, Json};
use chrono::{Datelike, Local};
use uuid::Uuid;
use ynab::types::AccountType;

use crate::{
    common::{update_month_net_totals, update_year_net_totals},
    db,
    domain::{FinancialResourceMonthly, Month, MonthNum, ResourceType},
    error::{AppError, HttpJsonAppResult},
    startup::AppState,
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

    let mut resources =
        db::get_financial_resources_of_month(&db_conn_pool, current_month, current_year)
            .await
            .map_err(AppError::from_sqlx)?;

    let accounts = ynab_client.get_accounts().await?;
    // TODO: Add scrapping of other accounts.
    // try_join!(
    //     // TODO: Make it more testable, not able to mock at the moment...
    //     budget_data_api::get_balance_celi_jeremy().map_err(AppError::from),
    //     budget_data_api::get_balance_celi_sandryne().map_err(AppError::from),
    // )?;
    let mut refreshed = vec![];
    let mut resources_balance: HashMap<&str, i64> = HashMap::new();
    // TODO: Find a better way than having those hard coded...
    let bank_accounts = "Comptes Bancaires";
    let credit_cards = "Cartes de Crédit";
    let mortgage = "Prêt Hypothécaire";
    let cars_loan = "Prêts Automobile";
    // let celi_jeremy = "CELI Jeremy";
    // let celi_sandryne = "CELI Sandryne";

    // resources_balance.insert(celi_jeremy, balance_celi_j);
    // resources_balance.insert(celi_sandryne, balance_celi_s);

    for account in accounts.iter().filter(|a| !a.closed && !a.deleted) {
        match account.account_type {
            AccountType::Mortgage => {
                resources_balance
                    .entry(mortgage)
                    .and_modify(|b| *b += account.balance)
                    .or_insert_with(|| account.balance);
            }
            AccountType::AutoLoan => {
                resources_balance
                    .entry(cars_loan)
                    .and_modify(|b| *b += account.balance)
                    .or_insert_with(|| account.balance);
            }
            AccountType::CreditCard => {
                resources_balance
                    .entry(credit_cards)
                    .and_modify(|b| *b += account.balance)
                    .or_insert_with(|| account.balance);
            }
            AccountType::Checking | AccountType::Savings => {
                resources_balance
                    .entry(bank_accounts)
                    .and_modify(|b| *b += account.balance)
                    .or_insert_with(|| account.balance);
            }
            _ => (),
        }
    }

    for (name, balance) in &resources_balance {
        // TODO: Use ynab account id...
        if let Some(fr) = resources.iter_mut().find(|r| r.base.name == **name) {
            if *balance != fr.balance {
                fr.balance = *balance;
                refreshed.push(fr.base.id);
            }
        } else {
            let res = match *name {
                "Comptes Bancaires" => {
                    let mut new_res = FinancialResourceMonthly::new_asset(
                        name.to_string(),
                        ResourceType::Cash,
                        false,
                        current_month,
                        current_year,
                    );
                    new_res.balance = *balance;
                    new_res
                }
                "Cartes de Crédit" => {
                    let mut new_res = FinancialResourceMonthly::new_liability(
                        name.to_string(),
                        ResourceType::Cash,
                        false,
                        current_month,
                        current_year,
                    );
                    new_res.balance = *balance;
                    new_res
                }
                "Prêt Hypothécaire" => {
                    let mut new_res = FinancialResourceMonthly::new_liability(
                        name.to_string(),
                        ResourceType::LongTerm,
                        false,
                        current_month,
                        current_year,
                    );
                    new_res.balance = *balance;
                    new_res
                }
                "Prêts Automobile" => {
                    let mut new_res = FinancialResourceMonthly::new_liability(
                        name.to_string(),
                        ResourceType::LongTerm,
                        false,
                        current_month,
                        current_year,
                    );
                    new_res.balance = *balance;
                    new_res
                }
                _ => unreachable!(),
            };
            refreshed.push(res.base.id);
            resources.push(res);
        }
    }

    if !refreshed.is_empty() {
        resources.retain(|r| refreshed.contains(&r.base.id));
        // TODO: Make sure month exists before updating financial resource.
        for r in resources {
            db::update_monthly_financial_resource(&db_conn_pool, &r).await?;
        }
        update_month_net_totals(&db_conn_pool, current_month, current_year).await?;
        update_year_net_totals(&db_conn_pool, current_year).await?;
    }

    Ok(Json(refreshed))
}
