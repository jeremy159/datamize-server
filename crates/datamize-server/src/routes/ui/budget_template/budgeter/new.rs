use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::State, response::Redirect};
use axum_extra::extract::Form;
use datamize_domain::{SaveBudgeterConfig, Uuid};
use serde::Deserialize;
use ynab::Payee;

use crate::{
    error::DatamizeResult,
    services::{budget_providers::DynYnabPayeeService, budget_template::DynBudgeterService},
};

pub async fn get(
    State((_, ynab_payee_service)): State<(DynBudgeterService, DynYnabPayeeService)>,
) -> DatamizeResult<impl IntoResponse> {
    let ynab_payees = ynab_payee_service.get_all_ynab_payees().await?;
    Ok(NewBudgeterTemplate {
        ynab_payees,
        name: "".to_string(),
        payees: vec![],
        error: None,
    })
}

#[derive(Template)]
#[template(path = "partials/budgeter/new.html")]
struct NewBudgeterTemplate {
    ynab_payees: Vec<Payee>,
    name: String,
    payees: Vec<Uuid>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct Payload {
    name: String,
    #[serde(rename = "payee")]
    payees: Vec<Uuid>,
}

pub async fn post(
    State((budgeter_service, ynab_payee_service)): State<(DynBudgeterService, DynYnabPayeeService)>,
    Form(payload): Form<Payload>,
) -> DatamizeResult<impl IntoResponse> {
    let new_budgeter = SaveBudgeterConfig {
        name: payload.name,
        payee_ids: payload.payees,
    };
    match budgeter_service.create_budgeter(new_budgeter.clone()).await {
        Ok(_) => Ok(Redirect::to("/budget/summary").into_response()),
        Err(e) => {
            let ynab_payees = ynab_payee_service.get_all_ynab_payees().await?;
            Ok(NewBudgeterTemplate {
                ynab_payees,
                name: new_budgeter.name.clone(),
                payees: new_budgeter.payee_ids,
                error: Some(e.to_string()),
            }
            .into_response())
        }
    }
}
