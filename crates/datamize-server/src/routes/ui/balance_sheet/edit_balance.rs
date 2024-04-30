use std::collections::BTreeMap;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use axum_extra::extract::Form;
use datamize_domain::{BalancePerMonth, MonthNum, Uuid, YearlyBalances};
use serde::Deserialize;

use crate::{
    error::DatamizeResult,
    routes::ui::{num_to_currency, num_to_currency_rounded},
    services::balance_sheet::DynFinResService,
};

pub async fn get(
    Path((year, month, fin_res_id)): Path<(i32, MonthNum, Uuid)>,
    State(fin_res_service): State<DynFinResService>,
) -> DatamizeResult<impl IntoResponse> {
    let fin_res = fin_res_service.get_fin_res(fin_res_id).await?;
    let balance = fin_res.get_balance(year, month);
    Ok(YearDetailsBalanceFormTemplate {
        fin_res_id,
        year,
        month,
        balance,
    })
}

#[derive(Template)]
#[template(path = "partials/year-details/edit-single-balance.html")]
struct YearDetailsBalanceFormTemplate {
    fin_res_id: Uuid,
    year: i32,
    month: MonthNum,
    balance: Option<i64>,
}

pub async fn put(
    Path((year, month, fin_res_id)): Path<(i32, MonthNum, Uuid)>,
    State(fin_res_service): State<DynFinResService>,
    Form(payload): Form<Payload>,
) -> DatamizeResult<impl IntoResponse> {
    let mut fin_res = fin_res_service.get_fin_res(fin_res_id).await?;
    // clear other balances to only update net assets of modified month
    fin_res.clear_all_balances();
    let balance: BalancePerMonth =
        BTreeMap::from([(month, payload.balance.map(|b| (b * 1000_f64) as i64))]);
    fin_res.insert_balance_for_year(year, balance);
    let fin_res = fin_res_service.update_fin_res(fin_res).await?;

    Ok((
        [("Hx-Trigger", "balance-updated")],
        YearDetailsSingleBalanceTemplate {
            fin_res_id,
            year,
            month,
            balance: fin_res.get_balance(year, month),
        },
    ))
}

#[derive(Deserialize)]
pub struct Payload {
    balance: Option<f64>,
}

#[derive(Template)]
#[template(path = "partials/year-details/single-balance.html")]
struct YearDetailsSingleBalanceTemplate {
    fin_res_id: Uuid,
    year: i32,
    month: MonthNum,
    balance: Option<i64>,
}
