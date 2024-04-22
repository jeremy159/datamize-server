use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use datamize_domain::{get_all_months_empty, BalancePerYearPerMonth, YearlyBalances};

use crate::{
    error::DatamizeResult,
    routes::ui::{num_to_currency, num_to_currency_rounded},
    services::balance_sheet::{DynFinResService, DynMonthService},
};

pub async fn get(
    Path(year): Path<i32>,
    State((_, fin_res_service)): State<(DynMonthService, DynFinResService)>,
) -> DatamizeResult<impl IntoResponse> {
    let mut fin_res = fin_res_service.get_all_fin_res_from_year(year).await?;
    let empty_months = get_all_months_empty();
    let mut total_assets = TotalRow::default();

    fin_res.retain(|fr| fr.base.resource_type.is_asset());

    for fin_res in &fin_res {
        for (year, month, balance) in fin_res.iter_balances() {
            match total_assets.get_balance(year, month) {
                Some(total_balance) => {
                    total_assets.insert_balance(year, month, total_balance + balance);
                }
                None => {
                    total_assets.insert_balance(year, month, balance);
                }
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

    Ok(YearDetailsTotalAssetsTemplate { total_assets })
}

#[derive(Template)]
#[template(path = "partials/year-details/total-assets.html")]
struct YearDetailsTotalAssetsTemplate {
    total_assets: TotalRow,
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
