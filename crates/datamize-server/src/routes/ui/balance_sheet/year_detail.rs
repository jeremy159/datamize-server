pub mod total_assets;
pub mod total_liabilities;
pub mod total_monthly;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use datamize_domain::{
    get_all_months_empty, BalancePerYearPerMonth, FinancialResourceYearly, Month, ResourceCategory,
    YearlyBalances,
};

use crate::{
    error::DatamizeResult,
    routes::ui::{num_to_currency, num_to_currency_rounded, num_to_percentage_f32},
    services::balance_sheet::{DynFinResService, DynMonthService},
};

pub async fn get(
    Path(year): Path<i32>,
    State((month_service, fin_res_service)): State<(DynMonthService, DynFinResService)>,
) -> DatamizeResult<impl IntoResponse> {
    let mut fin_res = fin_res_service.get_all_fin_res_from_year(year).await?;
    let empty_months = get_all_months_empty();
    for fin_res in &mut fin_res {
        match fin_res.get_balance_for_year(year) {
            Some(current_year_balances) => {
                if current_year_balances.len() < 12 {
                    fin_res.insert_balance_for_year(year, empty_months.clone());
                    for (m, b) in current_year_balances {
                        fin_res.insert_balance_opt(year, m, b);
                    }
                }
            }
            None => {
                fin_res.insert_balance_for_year(year, empty_months.clone());
            }
        }
    }
    let months = month_service.get_all_months_from_year(year).await?;

    let mut total_assets = TotalRow::default();
    let mut total_liabilities = TotalRow::default();

    for fin_res in &fin_res {
        for (year, month, balance) in fin_res.iter_balances() {
            match fin_res.base.resource_type.category() {
                ResourceCategory::Asset => match total_assets.get_balance(year, month) {
                    Some(total_balance) => {
                        total_assets.insert_balance(year, month, total_balance + balance);
                    }
                    None => {
                        total_assets.insert_balance(year, month, balance);
                    }
                },
                ResourceCategory::Liability => match total_liabilities.get_balance(year, month) {
                    Some(total_balance) => {
                        total_liabilities.insert_balance(year, month, total_balance + balance);
                    }
                    None => {
                        total_liabilities.insert_balance(year, month, balance);
                    }
                },
            }
        }
    }

    match total_assets.get_balance_for_year(year) {
        Some(current_year_balances) => {
            if current_year_balances.len() < 12 {
                total_assets.insert_balance_for_year(year, empty_months.clone());
                for (m, b) in current_year_balances {
                    total_assets.insert_balance_opt(year, m, b);
                }
            }
        }
        None => {
            total_assets.insert_balance_for_year(year, empty_months.clone());
        }
    }

    match total_liabilities.get_balance_for_year(year) {
        Some(current_year_balances) => {
            if current_year_balances.len() < 12 {
                total_liabilities.insert_balance_for_year(year, empty_months.clone());
                for (m, b) in current_year_balances {
                    total_liabilities.insert_balance_opt(year, m, b);
                }
            }
        }
        None => {
            total_liabilities.insert_balance_for_year(year, empty_months.clone());
        }
    }

    Ok(YearDetailsTemplate {
        year,
        months,
        fin_res,
        total_assets,
        total_liabilities,
    })
}

#[derive(Template)]
#[template(path = "pages/year-details.html")]
struct YearDetailsTemplate {
    year: i32,
    months: Vec<Month>,
    fin_res: Vec<FinancialResourceYearly>,
    total_assets: TotalRow,
    total_liabilities: TotalRow,
}

#[derive(Debug, Clone, Default)]
struct TotalRow {
    balances: BalancePerYearPerMonth,
}

impl YearlyBalances for TotalRow {
    fn balances(&self) -> &BalancePerYearPerMonth {
        &self.balances
    }

    fn balances_mut(&mut self) -> &mut BalancePerYearPerMonth {
        &mut self.balances
    }
}
