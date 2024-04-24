use std::collections::BTreeMap;

use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    response::Redirect,
};
use axum_extra::extract::Form;
use datamize_domain::{
    get_res_cat_options, get_res_type_options, BalancePerMonth, BaseFinancialResource,
    FinancialResourceYearly, MonthNum, ResourceCategory, ResourceCategoryOption,
    ResourceTypeOption, Uuid, YearlyBalances,
};
use serde::Deserialize;

use crate::{
    error::DatamizeResult,
    services::{
        balance_sheet::DynFinResService,
        budget_providers::{DynExternalAccountService, DynYnabAccountService},
    },
};

#[derive(Template)]
#[template(path = "pages/financial-resource/edit.html")]
struct FinancialResourceFormTemplate {
    fin_res: FinancialResourceYearly,
    year: i32,
    resource_categories: [ResourceCategoryOption; 2],
    resource_types: [ResourceTypeOption; 3],
    balances: Option<BalancePerMonth>,
    ynab_accounts: Vec<ynab::Account>,
    selected_ynab_accounts: Vec<Uuid>,
    external_accounts: Vec<datamize_domain::ExternalAccount>,
    selected_external_accounts: Vec<Uuid>,
    error: Option<String>,
}

impl FinancialResourceFormTemplate {
    async fn build(
        fin_res: FinancialResourceYearly,
        year: i32,
        error: Option<String>,
        ynab_account_service: DynYnabAccountService,
        external_account_service: DynExternalAccountService,
    ) -> DatamizeResult<Self> {
        let balances = fin_res.get_balance_for_year(year);
        let ynab_accounts: Vec<ynab::Account> =
            ynab_account_service.get_all_ynab_accounts().await?;
        let selected_ynab_accounts = fin_res.base.ynab_account_ids.clone().unwrap_or_default();
        let selected_external_accounts = fin_res
            .base
            .external_account_ids
            .clone()
            .unwrap_or_default();

        let external_accounts: Vec<datamize_domain::ExternalAccount> =
            external_account_service.get_all_external_accounts().await?;

        let resource_categories = get_res_cat_options(&fin_res.base.resource_type);
        // TODO: Populate resource type based on resource category. check https://htmx.org/examples/value-select/
        let resource_types = get_res_type_options(&fin_res.base.resource_type);

        Ok(Self {
            fin_res,
            year,
            resource_categories,
            resource_types,
            balances,
            ynab_accounts,
            selected_ynab_accounts,
            external_accounts,
            selected_external_accounts,
            error,
        })
    }
}

pub async fn get(
    Path((year, fin_res_id)): Path<(i32, Uuid)>,
    State((fin_res_service, ynab_account_service, external_account_service)): State<(
        DynFinResService,
        DynYnabAccountService,
        DynExternalAccountService,
    )>,
) -> DatamizeResult<impl IntoResponse> {
    let fin_res = fin_res_service.get_fin_res(fin_res_id).await?;
    FinancialResourceFormTemplate::build(
        fin_res,
        year,
        None,
        ynab_account_service,
        external_account_service,
    )
    .await
}

pub async fn put(
    Path((year, fin_res_id)): Path<(i32, Uuid)>,
    State((fin_res_service, ynab_account_service, external_account_service)): State<(
        DynFinResService,
        DynYnabAccountService,
        DynExternalAccountService,
    )>,
    Form(payload): Form<Payload>,
) -> DatamizeResult<impl IntoResponse> {
    let mut fin_res = FinancialResourceYearly {
        base: BaseFinancialResource {
            id: fin_res_id,
            name: payload.name,
            resource_type: format!("{}_{}", payload.category, payload.resource_type)
                .parse()
                .unwrap(),
            ynab_account_ids: payload.ynab_account_ids,
            external_account_ids: payload.external_account_ids,
        },
        balances: Default::default(),
    };
    let balances: BalancePerMonth = BTreeMap::from([
        (
            MonthNum::January,
            payload.january.map(|b| (b * 1000_f64) as i64),
        ),
        (
            MonthNum::February,
            payload.february.map(|b| (b * 1000_f64) as i64),
        ),
        (
            MonthNum::March,
            payload.march.map(|b| (b * 1000_f64) as i64),
        ),
        (
            MonthNum::April,
            payload.april.map(|b| (b * 1000_f64) as i64),
        ),
        (MonthNum::May, payload.may.map(|b| (b * 1000_f64) as i64)),
        (MonthNum::June, payload.june.map(|b| (b * 1000_f64) as i64)),
        (MonthNum::July, payload.july.map(|b| (b * 1000_f64) as i64)),
        (
            MonthNum::August,
            payload.august.map(|b| (b * 1000_f64) as i64),
        ),
        (
            MonthNum::September,
            payload.september.map(|b| (b * 1000_f64) as i64),
        ),
        (
            MonthNum::October,
            payload.october.map(|b| (b * 1000_f64) as i64),
        ),
        (
            MonthNum::November,
            payload.november.map(|b| (b * 1000_f64) as i64),
        ),
        (
            MonthNum::December,
            payload.december.map(|b| (b * 1000_f64) as i64),
        ),
    ]);
    fin_res.insert_balance_for_year(year, balances.clone());

    match fin_res_service.update_fin_res(fin_res.clone()).await {
        Ok(_) => Ok(Redirect::to(&format!(
            "/balance_sheet/years/{}/{}",
            year, fin_res.base.id
        ))
        .into_response()),
        Err(e) => Ok(FinancialResourceFormTemplate::build(
            fin_res,
            year,
            Some(e.to_string()),
            ynab_account_service,
            external_account_service,
        )
        .await?
        .into_response()),
    }
}

#[derive(Deserialize)]
pub struct Payload {
    name: String,
    category: ResourceCategory,
    #[serde(rename = "type")]
    resource_type: String,
    ynab_account_ids: Option<Vec<Uuid>>,
    external_account_ids: Option<Vec<Uuid>>,
    january: Option<f64>,
    february: Option<f64>,
    march: Option<f64>,
    april: Option<f64>,
    may: Option<f64>,
    june: Option<f64>,
    july: Option<f64>,
    august: Option<f64>,
    september: Option<f64>,
    october: Option<f64>,
    november: Option<f64>,
    december: Option<f64>,
}
