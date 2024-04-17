use askama_axum::IntoResponse;
use axum::{
    extract::{Path, State},
    response::Redirect,
};
use datamize_domain::Uuid;

use crate::services::{budget_providers::DynYnabPayeeService, budget_template::DynBudgeterService};

pub async fn delete(
    Path(id): Path<Uuid>,
    State((budgeter_service, _)): State<(DynBudgeterService, DynYnabPayeeService)>,
) -> impl IntoResponse {
    match budgeter_service.delete_budgeter(id).await {
        Ok(_) => Redirect::to("/budget/summary").into_response(),
        Err(e) => e.to_string().into_response(),
    }
}
