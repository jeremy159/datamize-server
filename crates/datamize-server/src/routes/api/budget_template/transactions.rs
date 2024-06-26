use axum::extract::State;
use datamize_domain::ScheduledTransactionsDistribution;

use crate::{
    error::{AppJson, HttpJsonDatamizeResult},
    services::budget_template::DynTemplateTransactionService,
};

/// Returns a budget template transactions, i.e. all the scheduled transactions in the upcoming 30 days.
pub async fn template_transactions(
    State(template_transaction_service): State<DynTemplateTransactionService>,
) -> HttpJsonDatamizeResult<ScheduledTransactionsDistribution> {
    Ok(AppJson(
        template_transaction_service
            .get_template_transactions()
            .await?,
    ))
}
