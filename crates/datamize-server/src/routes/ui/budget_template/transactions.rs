use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use datamize_domain::ScheduledTransactionsDistribution;

use crate::{
    error::{AppJson, HttpJsonDatamizeResult},
    services::budget_template::DynTemplateTransactionService,
};

/// Returns a budget template transactions, i.e. all the scheduled transactions in the upcoming 30 days.
pub async fn template_transactions(
    State(template_transaction_service): State<DynTemplateTransactionService>,
) -> impl IntoResponse {
    TransactionsTemplate {}
}

#[derive(Template)]
#[template(path = "pages/budget-transactions.html")]
struct TransactionsTemplate;
