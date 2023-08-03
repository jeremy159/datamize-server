use axum::{extract::State, Json};

use crate::{
    error::HttpJsonDatamizeResult, models::budget_template::ScheduledTransactionsDistribution,
    services::budget_template::TemplateTransactionServiceExt,
};

/// Returns a budget template transactions, i.e. all the scheduled transactions in the upcoming 30 days.
pub async fn template_transactions<TTS: TemplateTransactionServiceExt>(
    State(mut template_transaction_service): State<TTS>,
) -> HttpJsonDatamizeResult<ScheduledTransactionsDistribution> {
    Ok(Json(
        template_transaction_service
            .get_template_transactions()
            .await?,
    ))
}
