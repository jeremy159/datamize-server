use std::sync::Arc;

use chrono::{Datelike, Local, NaiveDate};
use datamize_domain::{
    async_trait,
    db::ynab::{DynYnabScheduledTransactionMetaRepo, DynYnabScheduledTransactionRepo},
    DatamizeScheduledTransaction,
};
use ynab::ScheduledTransactionRequests;

use crate::error::DatamizeResult;

#[async_trait]
pub trait ScheduledTransactionServiceExt: Send + Sync {
    async fn get_latest_scheduled_transactions(
        &self,
    ) -> DatamizeResult<Vec<DatamizeScheduledTransaction>>;
}

pub type DynScheduledTransactionService = Arc<dyn ScheduledTransactionServiceExt>;

#[derive(Clone)]
pub struct ScheduledTransactionService {
    pub ynab_scheduled_transaction_repo: DynYnabScheduledTransactionRepo,
    pub ynab_scheduled_transaction_meta_repo: DynYnabScheduledTransactionMetaRepo,
    pub ynab_client: Arc<dyn ScheduledTransactionRequests + Send + Sync>,
}

impl ScheduledTransactionService {
    pub fn new_arced(
        ynab_scheduled_transaction_repo: DynYnabScheduledTransactionRepo,
        ynab_scheduled_transaction_meta_repo: DynYnabScheduledTransactionMetaRepo,
        ynab_client: Arc<dyn ScheduledTransactionRequests + Send + Sync>,
    ) -> Arc<Self> {
        Arc::new(ScheduledTransactionService {
            ynab_scheduled_transaction_repo,
            ynab_scheduled_transaction_meta_repo,
            ynab_client,
        })
    }

    pub(crate) async fn check_last_saved(&self) -> DatamizeResult<()> {
        let current_date = Local::now().date_naive();
        if let Ok(last_saved) = self
            .ynab_scheduled_transaction_meta_repo
            .get_last_saved()
            .await
        {
            let last_saved_date: NaiveDate = last_saved.parse()?;
            if current_date.month() != last_saved_date.month() {
                tracing::debug!(
                    ?current_date,
                    ?last_saved_date,
                    "discarding knowledge_server",
                );
                // Discard knowledge_server when changing month.
                self.ynab_scheduled_transaction_meta_repo
                    .del_delta()
                    .await?;
                self.ynab_scheduled_transaction_meta_repo
                    .set_last_saved(current_date.to_string())
                    .await?;
            }
        } else {
            self.ynab_scheduled_transaction_meta_repo
                .set_last_saved(current_date.to_string())
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl ScheduledTransactionServiceExt for ScheduledTransactionService {
    #[tracing::instrument(skip(self))]
    async fn get_latest_scheduled_transactions(
        &self,
    ) -> DatamizeResult<Vec<DatamizeScheduledTransaction>> {
        self.check_last_saved().await?;
        let saved_scheduled_transactions_delta = self
            .ynab_scheduled_transaction_meta_repo
            .get_delta()
            .await
            .ok();

        let scheduled_transactions_delta = self
            .ynab_client
            .get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
            .await?;

        self.ynab_scheduled_transaction_repo
            .update_all(&scheduled_transactions_delta.scheduled_transactions)
            .await?;

        self.ynab_scheduled_transaction_meta_repo
            .set_delta(scheduled_transactions_delta.server_knowledge)
            .await?;

        Ok(self
            .ynab_scheduled_transaction_repo
            .get_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
