use std::collections::HashMap;

use axum::{extract::State, Json};
use chrono::{Datelike, Local};
use futures::{try_join, TryFutureExt};
use uuid::Uuid;
use ynab::types::AccountType;

use crate::{
    common::get_month,
    db,
    domain::MonthNum,
    error::{AppError, HttpJsonAppResult},
    startup::AppState,
};

/// Endpoint to refresh non-editable financial resources.
/// Only resources from the current month will be refreshed by this endpoint.
/// This endpoint basically calls the YNAB api for some resources and starts a web scrapper for others.
/// Will return an array of ids for Financial Resources updated.
#[tracing::instrument(skip_all)]
pub async fn refresh_balance_sheet_resources(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Uuid>> {
    let db_conn_pool = app_state.db_conn_pool;
    let ynab_client = app_state.ynab_client.as_ref();
    let current_date = Local::now().date_naive();

    let Some(year_data) = db::get_year_data(&db_conn_pool, current_date.year())
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    // TODO: Add scrapping of other accounts.
    let (accounts, balance_celi_j, balance_celi_s, mut month) = try_join!(
        ynab_client.get_accounts().map_err(AppError::from),
        budget_data_api::get_balance_celi_jeremy().map_err(AppError::from),
        budget_data_api::get_balance_celi_sandryne().map_err(AppError::from),
        get_month(
            &db_conn_pool,
            year_data.id,
            current_date.month().try_into().unwrap()
        )
    )?;
    let mut refreshed = vec![];
    let mut resources_balance: HashMap<&str, i64> = HashMap::new();
    // TODO: Find a better way than having those hard coded...
    let bank_accounts = "Comptes Bancaires";
    let credit_cards = "Cartes de Crédit";
    let mortgage = "Prêt Hypothécaire";
    let cars_loan = "Prêts Automobile";
    let celi_jeremy = "CELI Jeremy";
    let celi_sandryne = "CELI Sandryne";

    resources_balance.insert(celi_jeremy, balance_celi_j);
    resources_balance.insert(celi_sandryne, balance_celi_s);

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
        if let Some(fr) = month.resources.iter_mut().find(|r| r.name == **name) {
            if *balance != fr.balance {
                fr.balance = *balance;
                refreshed.push(fr.id);
            }
        }
    }

    if !refreshed.is_empty() {
        db::update_financial_resources(&db_conn_pool, &month.resources).await?;
        month.compute_net_totals();

        let year_data_opt = match month.month.pred() {
            MonthNum::December => db::get_year_data(&db_conn_pool, current_date.year() - 1).await,
            _ => Ok(Some(year_data)),
        };

        if let Ok(Some(year_data)) = year_data_opt {
            if let Ok(Some(prev_month)) =
                db::get_month_data(&db_conn_pool, year_data.id, month.month.pred() as i16).await
            {
                if let Ok(prev_net_totals) =
                    db::get_month_net_totals_for(&db_conn_pool, prev_month.id).await
                {
                    month.update_net_totals_with_previous(&prev_net_totals);
                }
            }
        }

        db::update_month_net_totals(&db_conn_pool, &month.net_totals).await?;
    }

    Ok(Json(refreshed))
}
