use axum::{extract::State, Json};

use crate::{
    error::HttpJsonDatamizeResult,
    models::budget_template::ScheduledTransactionsDistribution,
    services::budget_template::{TemplateTransactionService, TemplateTransactionServiceExt},
};

/// Returns a budget template transactions, i.e. all the scheduled transactions in the upcoming 30 days.
pub async fn template_transactions(
    State(mut template_transaction_service): State<TemplateTransactionService>,
) -> HttpJsonDatamizeResult<ScheduledTransactionsDistribution> {
    Ok(Json(
        template_transaction_service
            .get_template_transactions()
            .await?,
    ))
}
