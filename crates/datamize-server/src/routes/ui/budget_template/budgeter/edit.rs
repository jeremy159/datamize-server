use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    response::Redirect,
};
use axum_extra::extract::Form;
use datamize_domain::{BudgeterConfig, Uuid};
use serde::Deserialize;
use ynab::Payee;

use crate::{
    error::DatamizeResult,
    services::{budget_providers::DynYnabPayeeService, budget_template::DynBudgeterService},
};

pub async fn get(
    Path(id): Path<Uuid>,
    State((budgeter_service, ynab_payee_service)): State<(DynBudgeterService, DynYnabPayeeService)>,
) -> DatamizeResult<impl IntoResponse> {
    let budgeter = budgeter_service.get_budgeter(id).await?;
    let ynab_payees = ynab_payee_service.get_all_ynab_payees().await?;
    Ok(EditBudgeterTemplate {
        ynab_payees,
        id: budgeter.id,
        name: budgeter.name.clone(),
        payees: budgeter.payee_ids,
        error: None,
    })
}

#[derive(Template)]
#[template(path = "partials/budgeter/edit.html")]
struct EditBudgeterTemplate {
    ynab_payees: Vec<Payee>,
    id: Uuid,
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
    Path(id): Path<Uuid>,
    State((budgeter_service, ynab_payee_service)): State<(DynBudgeterService, DynYnabPayeeService)>,
    Form(payload): Form<Payload>,
) -> DatamizeResult<impl IntoResponse> {
    let new_budgeter = BudgeterConfig {
        id,
        name: payload.name,
        payee_ids: payload.payees,
    };
    match budgeter_service.update_budgeter(new_budgeter.clone()).await {
        Ok(_) => Ok(Redirect::to("/budget/summary").into_response()),
        Err(e) => {
            let ynab_payees = ynab_payee_service.get_all_ynab_payees().await?;
            Ok(EditBudgeterTemplate {
                ynab_payees,
                id: new_budgeter.id,
                name: new_budgeter.name.clone(),
                payees: new_budgeter.payee_ids,
                error: Some(e.to_string()),
            }
            .into_response())
        }
    }
}
