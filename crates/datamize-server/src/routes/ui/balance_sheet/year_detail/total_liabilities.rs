use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use datamize_domain::{BalancePerYearPerMonth, YearlyBalances};

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
    let mut total_liabilities = TotalRow::default();

    fin_res.retain(|fr| fr.base.resource_type.is_liability());

    for fin_res in &fin_res {
        for (year, month, balance) in fin_res.iter_balances() {
            match total_liabilities.get_balance(year, month) {
                Some(total_balance) => {
                    total_liabilities.insert_balance(year, month, total_balance + balance);
                }
                None => {
                    total_liabilities.insert_balance(year, month, balance);
                }
            }
        }
    }

    Ok(YearDetailsTotalAssetsTemplate { total_liabilities })
}

#[derive(Template)]
#[template(path = "partials/year-details/total-liabilities.html")]
struct YearDetailsTotalAssetsTemplate {
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
