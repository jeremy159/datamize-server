use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate};
use ynab::Client;

use crate::{
    db::budget_providers::ynab::{YnabScheduledTransactionMetaRepo, YnabScheduledTransactionRepo},
    error::DatamizeResult,
    models::budget_template::DatamizeScheduledTransaction,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ScheduledTransactionServiceExt {
    async fn get_latest_scheduled_transactions(
        &mut self,
    ) -> DatamizeResult<Vec<DatamizeScheduledTransaction>>;
}

pub struct ScheduledTransactionService<
    YSTR: YnabScheduledTransactionRepo,
    YSTMR: YnabScheduledTransactionMetaRepo,
> {
    pub ynab_scheduled_transaction_repo: YSTR,
    pub ynab_scheduled_transaction_meta_repo: YSTMR,
    pub ynab_client: Arc<Client>,
}

#[async_trait]
impl<YSTR, YSTMR> ScheduledTransactionServiceExt for ScheduledTransactionService<YSTR, YSTMR>
where
    YSTR: YnabScheduledTransactionRepo + Sync + Send,
    YSTMR: YnabScheduledTransactionMetaRepo + Sync + Send,
{
    #[tracing::instrument(skip(self))]
    async fn get_latest_scheduled_transactions(
        &mut self,
    ) -> DatamizeResult<Vec<DatamizeScheduledTransaction>> {
        let current_date = Local::now().date_naive();
        if let Ok(last_saved) = self
            .ynab_scheduled_transaction_meta_repo
            .get_last_saved()
            .await
        {
            let last_saved_date: NaiveDate = last_saved.parse()?;
            if current_date.month() != last_saved_date.month() {
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
        let saved_scheduled_transactions_delta = self
            .ynab_scheduled_transaction_meta_repo
            .get_delta()
            .await
            .ok();

        let scheduled_transactions_delta = self
            .ynab_client
            .get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
            .await
            .context("failed to get scheduled transactions from ynab's API")?;

        self.ynab_scheduled_transaction_repo
            .update_all(&scheduled_transactions_delta.scheduled_transactions)
            .await
            .context("failed to save scheduled transactions in database")?;

        self.ynab_scheduled_transaction_meta_repo
            .set_delta(scheduled_transactions_delta.server_knowledge)
            .await
            .context(
                "failed to save last known server knowledge of scheduled transactions in redis",
            )?;

        Ok(self
            .ynab_scheduled_transaction_repo
            .get_all()
            .await
            .context("failed to get scheduled transactions from database")?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
