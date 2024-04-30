use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;

use crate::{error::DatamizeResult, services::budget_template::DynTemplateTransactionService};

/// Returns a budget template transactions, i.e. all the scheduled transactions in the upcoming 30 days.
pub async fn template_transactions(
    State(_template_transaction_service): State<DynTemplateTransactionService>,
) -> DatamizeResult<impl IntoResponse> {
    Ok(TransactionsTemplate {})
}

#[derive(Template)]
#[template(path = "pages/budget-transactions.html")]
struct TransactionsTemplate;
