use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::ynab::{DynYnabTransactionMetaRepo, DynYnabTransactionRepo},
    Uuid,
};
use ynab::{TransactionDetail, TransactionRequests};

use crate::error::DatamizeResult;

#[async_trait]
pub trait TransactionServiceExt: Send + Sync {
    async fn refresh_saved_transactions(&self) -> DatamizeResult<()>;
    async fn get_latest_transactions(&self) -> DatamizeResult<Vec<TransactionDetail>>;
    async fn get_transactions_by_category_id(
        &self,
        category_id: Uuid,
    ) -> DatamizeResult<Vec<TransactionDetail>>;
    async fn get_transactions_by_payee_id(
        &self,
        payee_id: Uuid,
    ) -> DatamizeResult<Vec<TransactionDetail>>;
}

pub type DynTransactionService = Arc<dyn TransactionServiceExt>;

#[derive(Clone)]
pub struct TransactionService {
    pub ynab_transaction_repo: DynYnabTransactionRepo,
    pub ynab_transaction_meta_repo: DynYnabTransactionMetaRepo,
    pub ynab_client: Arc<dyn TransactionRequests + Send + Sync>,
}

#[async_trait]
impl TransactionServiceExt for TransactionService {
    #[tracing::instrument(skip(self))]
    async fn refresh_saved_transactions(&self) -> DatamizeResult<()> {
        let saved_transactions_delta = self.ynab_transaction_meta_repo.get_delta().await.ok();

        let transactions_delta = self
            .ynab_client
            .get_transactions_delta(saved_transactions_delta)
            .await?;

        self.ynab_transaction_repo
            .update_all(&transactions_delta.transactions)
            .await?;

        self.ynab_transaction_meta_repo
            .set_delta(transactions_delta.server_knowledge)
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_latest_transactions(&self) -> DatamizeResult<Vec<TransactionDetail>> {
        self.refresh_saved_transactions().await?;

        Ok(self
            .ynab_transaction_repo
            .get_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get_transactions_by_category_id(
        &self,
        category_id: Uuid,
    ) -> DatamizeResult<Vec<TransactionDetail>> {
        Ok(self
            .ynab_transaction_repo
            .get_all_with_category_id(category_id)
            .await?)
    }

    #[tracing::instrument(skip(self))]
    async fn get_transactions_by_payee_id(
        &self,
        payee_id: Uuid,
    ) -> DatamizeResult<Vec<TransactionDetail>> {
        Ok(self
            .ynab_transaction_repo
            .get_all_with_payee_id(payee_id)
            .await?)
    }
}

impl TransactionService {
    pub fn new_arced(
        ynab_transaction_repo: DynYnabTransactionRepo,
        ynab_transaction_meta_repo: DynYnabTransactionMetaRepo,
        ynab_client: Arc<dyn TransactionRequests + Send + Sync>,
    ) -> Arc<Self> {
        Arc::new(TransactionService {
            ynab_transaction_repo,
            ynab_transaction_meta_repo,
            ynab_client,
        })
    }
}
