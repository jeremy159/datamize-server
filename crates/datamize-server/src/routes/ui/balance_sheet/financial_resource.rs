pub mod edit;
pub mod new;
pub mod refresh;
pub mod types;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use axum_extra::{extract::OptionalQuery, response::Html};
use chrono::{Datelike, Local};
use datamize_domain::{BalancePerMonth, FinancialResourceYearly, Uuid, YearQuery, YearlyBalances};
use http::HeaderMap;
use serde_json::json;

use crate::{
    error::DatamizeResult,
    routes::ui::num_to_currency,
    services::{
        balance_sheet::{DynFinResService, DynYearService},
        budget_providers::{DynExternalAccountService, DynYnabAccountService},
    },
};

/// View to simply look at the financial resource without doing any modifications
pub async fn get(
    Path(fin_res_id): Path<Uuid>,
    OptionalQuery(param): OptionalQuery<YearQuery>,
    State((fin_res_service, ynab_account_service, external_account_service, year_service)): State<
        (
            DynFinResService,
            DynYnabAccountService,
            DynExternalAccountService,
            DynYearService,
        ),
    >,
) -> DatamizeResult<impl IntoResponse> {
    let fin_res = fin_res_service.get_fin_res(fin_res_id).await?;
    let year = param.map_or(Local::now().date_naive().year(), |p| p.year);
    let balances = fin_res.get_balance_for_year(year);
    let mut ynab_accounts: Vec<ynab::Account> =
        ynab_account_service.get_all_ynab_accounts().await?;

    ynab_accounts.retain(|a| {
        fin_res
            .base
            .ynab_account_ids
            .clone()
            .map_or(false, |accounts| accounts.contains(&a.id))
    });

    let mut external_accounts: Vec<datamize_domain::ExternalAccount> =
        external_account_service.get_all_external_accounts().await?;

    external_accounts.retain(|a| {
        fin_res
            .base
            .external_account_ids
            .clone()
            .map_or(false, |accounts| accounts.contains(&a.id))
    });

    Ok(FinancialResourceTemplate {
        fin_res,
        year,
        balances,
        ynab_accounts,
        external_accounts,
        years: year_service.get_all_years_num().await?,
    })
}

#[derive(Template)]
#[template(path = "pages/financial-resource/view.html")]
struct FinancialResourceTemplate {
    fin_res: FinancialResourceYearly,
    year: i32,
    years: Vec<i32>,
    balances: Option<BalancePerMonth>,
    ynab_accounts: Vec<ynab::Account>,
    external_accounts: Vec<datamize_domain::ExternalAccount>,
}

pub async fn delete(
    Path(fin_res_id): Path<Uuid>,
    OptionalQuery(param): OptionalQuery<YearQuery>,
    State((fin_res_service, _, _, _)): State<(
        DynFinResService,
        DynYnabAccountService,
        DynExternalAccountService,
        DynYearService,
    )>,
    headers: HeaderMap,
) -> DatamizeResult<impl IntoResponse> {
    // No matter the result of the deletion, we either redirect or do nothing.
    _ = fin_res_service.delete_fin_res(fin_res_id).await;
    let year = param.map_or(Local::now().date_naive().year(), |p| p.year);

    Ok(match headers.get("Hx-Trigger") {
        Some(trigger) => {
            if trigger.to_str().unwrap() == "delete-btn" {
                [("Hx-Location", json!({"path": &format!("/balance_sheet/years/{}", year), "target": "#main", "swap": "outerHTML", "select": "#main"}).to_string())].into_response()
            } else {
                Html("").into_response()
            }
        }
        None => Html("").into_response(),
    })
}
