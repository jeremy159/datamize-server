use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    Json,
};

use ynab::types::AccountType;

use crate::{
    domain::{FinancialResource, Month, ResourceType, TotalSummary},
    error::HttpJsonAppResult,
    startup::AppState,
};

/// Get The details of one month
pub async fn get_balance_sheet_month(
    Path((year, month)): Path<(i32, chrono::Month)>,
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
                    .of_type(ResourceType::Investement)
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
                    .of_type(ResourceType::Investement)
                    .non_editable()
                    .with_balance(balance)
            });
    }

    let mut resources = resources.into_values().collect::<Vec<_>>();
    resources.push(
        FinancialResource::new_asset("REER Jeremy".to_string())
            .of_type(ResourceType::Investement)
            .with_balance(29809840),
    );
    resources.push(
        FinancialResource::new_asset("RPA Sandryne".to_string())
            .of_type(ResourceType::Investement)
            .with_balance(4545820),
    );
    resources.push(
        FinancialResource::new_asset("REEE".to_string())
            .of_type(ResourceType::Investement)
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
    let net_assets = TotalSummary {
        total: resources.iter().map(|v| v.balance).sum(),
        balance_variation: 2806000,
        percent_variation: 0.013,
    };
    let net_portfolio = TotalSummary {
        total: resources
            .iter()
            .filter(|v| {
                v.resource_type == ResourceType::Cash
                    || v.resource_type == ResourceType::Investement
            })
            .map(|v| v.balance)
            .sum(),
        balance_variation: 1200000,
        percent_variation: 0.021,
    };

    Ok(Json(Month {
        month,
        net_assets,
        net_portfolio,
        resources,
    }))
}

/// Updates the month
pub async fn put_balance_sheet_month(
    Path((year, month)): Path<(i32, chrono::Month)>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Month> {
    Ok(Json(Month {
        month,
        net_assets: TotalSummary {
            total: 0,
            percent_variation: 0.0,
            balance_variation: 0,
        },
        net_portfolio: TotalSummary {
            total: 0,
            percent_variation: 0.0,
            balance_variation: 0,
        },
        resources: vec![],
    }))
}
