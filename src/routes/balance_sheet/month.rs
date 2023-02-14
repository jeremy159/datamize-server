use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use ynab::types::AccountType;

use crate::{
    common::get_month,
    db,
    domain::{
        FinancialResource, Month, MonthNum, NetTotal, NetTotalType, ResourceType, UpdateMonth,
    },
    error::{AppError, HttpJsonAppResult},
    startup::AppState,
};

pub async fn balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Month> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(get_month(&db_conn_pool, year, month).await?))
}

// TODO: To recompute net totals of current month and year after update.
/// Updates the month, i.e. all the financial resources included in the month
pub async fn update_balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(app_state): State<AppState>,
    Json(body): Json<UpdateMonth>,
) -> HttpJsonAppResult<Month> {
    let db_conn_pool = app_state.db_conn_pool;

    let Some(year_data) = db::get_year_data(&db_conn_pool, year)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let Some(month_data) = db::get_month_data(&db_conn_pool, year_data.id, month as i16)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let net_totals = db::get_month_net_totals_for(&db_conn_pool, month_data.id).await?;

    db::update_financial_resources(&db_conn_pool, &body.resources).await?;

    Ok(Json(Month {
        id: month_data.id,
        month,
        net_totals,
        resources: body.resources,
    }))
}

// TODO: To refactor to an endpoint to refresh data. Otherwise makes requests really slow.
/// Get The details of one month, including updating non_editable fields.
pub async fn get_balance_sheet_month(
    Path((_year, month)): Path<(i32, MonthNum)>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Month> {
    let ynab_client = app_state.ynab_client.as_ref();
    println!("Month received = {:?}", month);
    let accounts = ynab_client.get_accounts().await?;
    let mut resources: HashMap<&str, FinancialResource> = HashMap::new();
    let bank_accounts = "Comptes Bancaires".to_string();
    let credit_cards = "Cartes de Crédit".to_string();
    let mortgage = "Prêt Hypothécaire".to_string();
    let cars_loan = "Prêts Automobile".to_string();
    let celi_jeremy = "CELI Jeremy".to_string();
    let celi_sandryne = "CELI Sandryne".to_string();

    for account in accounts.iter().filter(|a| !a.closed && !a.deleted) {
        match account.account_type {
            AccountType::Mortgage => {
                resources
                    .entry(&mortgage)
                    .and_modify(|v| v.add_to_balance(account.balance))
                    .or_insert_with(|| {
                        FinancialResource::new_liability(mortgage.clone())
                            .of_type(ResourceType::LongTerm)
                            .non_editable()
                            .with_balance(account.balance)
                    });
            }
            AccountType::AutoLoan => {
                resources
                    .entry(&cars_loan)
                    .and_modify(|v| v.add_to_balance(account.balance))
                    .or_insert_with(|| {
                        FinancialResource::new_liability(cars_loan.clone())
                            .of_type(ResourceType::LongTerm)
                            .non_editable()
                            .with_balance(account.balance)
                    });
            }
            AccountType::CreditCard => {
                resources
                    .entry(&credit_cards)
                    .and_modify(|v| v.add_to_balance(account.balance))
                    .or_insert_with(|| {
                        FinancialResource::new_liability(credit_cards.clone())
                            .of_type(ResourceType::Cash)
                            .non_editable()
                            .with_balance(account.balance)
                    });
            }
            AccountType::Checking | AccountType::Savings => {
                resources
                    .entry(&bank_accounts)
                    .and_modify(|v| v.add_to_balance(account.balance))
                    .or_insert_with(|| {
                        FinancialResource::new_asset(bank_accounts.clone())
                            .of_type(ResourceType::Cash)
                            .non_editable()
                            .with_balance(account.balance)
                    });
            }
            _ => (),
        }
    }

    // TODO: Maybe move elsewhere? Makes request reaaaaallyyyyyyyy slow...
    if let Ok(balance) = budget_data_api::get_balance_celi_jeremy().await {
        resources
            .entry(&celi_jeremy)
            .and_modify(|v| v.override_balance(balance)) // Erase any previous balance with what we received
            .or_insert_with(|| {
                FinancialResource::new_asset(celi_jeremy.clone())
                    .of_type(ResourceType::Investment)
                    .non_editable()
                    .with_balance(balance)
            });
    }

    // TODO: Maybe move elsewhere? Makes request reaaaaallyyyyyyyy slow...
    if let Ok(balance) = budget_data_api::get_balance_celi_sandryne().await {
        resources
            .entry(&celi_sandryne)
            .and_modify(|v| v.override_balance(balance)) // Erase any previous balance with what we received
            .or_insert_with(|| {
                FinancialResource::new_asset(celi_sandryne.clone())
                    .of_type(ResourceType::Investment)
                    .non_editable()
                    .with_balance(balance)
            });
    }

    let mut resources = resources.into_values().collect::<Vec<_>>();
    resources.push(
        FinancialResource::new_asset("REER Jeremy".to_string())
            .of_type(ResourceType::Investment)
            .with_balance(29809840),
    );
    resources.push(
        FinancialResource::new_asset("RPA Sandryne".to_string())
            .of_type(ResourceType::Investment)
            .with_balance(4545820),
    );
    resources.push(
        FinancialResource::new_asset("REEE".to_string())
            .of_type(ResourceType::Investment)
            .with_balance(0000),
    );
    resources.push(
        FinancialResource::new_asset("Valeur Maison".to_string())
            .of_type(ResourceType::LongTerm)
            .with_balance(505900000),
    );
    resources.push(
        FinancialResource::new_asset("Valeur Automobile".to_string())
            .of_type(ResourceType::LongTerm)
            .with_balance(10804000),
    );

    // TODO: To remove stub data...
    let net_assets = NetTotal {
        id: Uuid::new_v4(),
        net_type: NetTotalType::Asset,
        total: resources.iter().map(|v| v.balance).sum(),
        balance_var: 2806000,
        percent_var: 0.013,
    };
    let net_portfolio = NetTotal {
        id: Uuid::new_v4(),
        net_type: NetTotalType::Portfolio,
        total: resources
            .iter()
            .filter(|v| {
                v.resource_type == ResourceType::Cash || v.resource_type == ResourceType::Investment
            })
            .map(|v| v.balance)
            .sum(),
        balance_var: 1200000,
        percent_var: 0.021,
    };

    Ok(Json(Month {
        id: Uuid::new_v4(),
        month,
        net_totals: vec![net_assets, net_portfolio],
        resources,
    }))
}
