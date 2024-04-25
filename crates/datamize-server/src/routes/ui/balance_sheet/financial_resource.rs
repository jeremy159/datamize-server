pub mod edit;
pub mod new;
pub mod refresh;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use axum_extra::response::Html;
use datamize_domain::{BalancePerMonth, FinancialResourceYearly, Uuid, YearlyBalances};
use http::HeaderMap;
use serde_json::json;

use crate::{
    error::DatamizeResult,
    routes::ui::num_to_currency,
    services::{
        balance_sheet::DynFinResService,
        budget_providers::{DynExternalAccountService, DynYnabAccountService},
    },
};

/// View to simply look at the financial resource without doing any modifications
pub async fn get(
    Path((year, fin_res_id)): Path<(i32, Uuid)>,
    State((fin_res_service, ynab_account_service, external_account_service)): State<(
        DynFinResService,
        DynYnabAccountService,
        DynExternalAccountService,
    )>,
) -> DatamizeResult<impl IntoResponse> {
    let fin_res = fin_res_service.get_fin_res(fin_res_id).await?;
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
        // TODO: Get all year numbers. Make the method in YearService.
    })
}

#[derive(Template)]
#[template(path = "pages/financial-resource/view.html")]
struct FinancialResourceTemplate {
    fin_res: FinancialResourceYearly,
    year: i32,
    balances: Option<BalancePerMonth>,
    ynab_accounts: Vec<ynab::Account>,
    external_accounts: Vec<datamize_domain::ExternalAccount>,
}

pub async fn delete(
    Path((year, fin_res_id)): Path<(i32, Uuid)>,
    State((fin_res_service, _, _)): State<(
        DynFinResService,
        DynYnabAccountService,
        DynExternalAccountService,
    )>,
    headers: HeaderMap,
) -> DatamizeResult<impl IntoResponse> {
    // No matter the result of the deletion, we either redirect or do nothing.
    _ = fin_res_service.delete_fin_res(fin_res_id).await;

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
