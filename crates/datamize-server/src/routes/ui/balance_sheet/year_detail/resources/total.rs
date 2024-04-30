use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use datamize_domain::{ResourceCategory, YearlyBalances};

use crate::{
    error::DatamizeResult,
    routes::ui::{balance_sheet::year_detail::TotalRow, num_to_currency, num_to_currency_rounded},
    services::balance_sheet::DynFinResService,
};

pub async fn get(
    Path((year, category)): Path<(i32, ResourceCategory)>,
    State(fin_res_service): State<DynFinResService>,
) -> DatamizeResult<impl IntoResponse> {
    let resources = fin_res_service
        .get_from_year_and_category(year, &category)
        .await?;
    let mut total_row = TotalRow::default();

    for fin_res in &resources {
        for (year, month, balance) in fin_res.iter_balances() {
            match total_row.get_balance(year, month) {
                Some(total_balance) => {
                    total_row.insert_balance(year, month, total_balance + balance);
                }
                None => {
                    total_row.insert_balance(year, month, balance);
                }
            }
        }
    }

    Ok(YearDetailsTotalAssetsTemplate { total_row })
}

#[derive(Template)]
#[template(path = "partials/year-details/total-row.html")]
struct YearDetailsTotalAssetsTemplate {
    total_row: TotalRow,
}
