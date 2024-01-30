use axum::{extract::State, Json};
use datamize_domain::ScheduledTransactionsDistribution;

use crate::{
    error::HttpJsonDatamizeResult, services::budget_template::DynTemplateTransactionService,
};

/// Returns a budget template transactions, i.e. all the scheduled transactions in the upcoming 30 days.
pub async fn template_transactions(
    State(template_transaction_service): State<DynTemplateTransactionService>,
) -> HttpJsonDatamizeResult<ScheduledTransactionsDistribution> {
    Ok(Json(
        template_transaction_service
            .get_template_transactions()
            .await?,
    ))
}
