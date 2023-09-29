use std::sync::Arc;

use async_trait::async_trait;
use mockall::mock;
use ynab::{
    SaveTransaction, TransactionDetail, TransactionRequests, TransactionType,
    TransationsDetailDelta, UpdateTransaction, YnabResult,
};

use crate::{
    db::{
        balance_sheet::MockSavingRateRepo,
        budget_providers::ynab::{MockYnabTransactionMetaRepoImpl, MockYnabTransactionRepoImpl},
    },
    services::{
        balance_sheet::{DynSavingRateService, SavingRateService},
        budget_providers::TransactionService,
    },
};

pub(crate) fn saving_rate_service() -> (
    DynSavingRateService,
    Arc<MockSavingRateRepo>,
    Box<MockYnabTransactionRepoImpl>,
    Box<MockYnabTransactionMetaRepoImpl>,
    Arc<dyn TransactionRequests + Send + Sync>,
) {
    let saving_rate_repo = Arc::new(MockSavingRateRepo::new());
    let ynab_transaction_repo = Box::new(MockYnabTransactionRepoImpl::new());
    let ynab_transaction_meta_repo = Box::new(MockYnabTransactionMetaRepoImpl::new());

    mock! {
        YnabClient {}
        #[async_trait]
        impl TransactionRequests for YnabClient {
            async fn get_transactions(&self) -> YnabResult<Vec<TransactionDetail>>;

            async fn get_transactions_delta(
                &self,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<TransationsDetailDelta>;

            async fn get_transactions_since(&self, since_date: &str) -> YnabResult<Vec<TransactionDetail>>;

            async fn get_transactions_since_delta(
                &self,
                since_date: &str,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<TransationsDetailDelta>;

            async fn get_transactions_of(
                &self,
                transaction_type: TransactionType,
            ) -> YnabResult<Vec<TransactionDetail>>;

            async fn get_transactions_of_delta(
                &self,
                transaction_type: TransactionType,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<TransationsDetailDelta>;

            async fn get_transactions_since_date_of(
                &self,
                since_date: &str,
                transaction_type: TransactionType,
            ) -> YnabResult<Vec<TransactionDetail>>;

            async fn get_transactions_since_date_of_delta(
                &self,
                since_date: &str,
                transaction_type: TransactionType,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<TransationsDetailDelta>;

            async fn get_transactions_by_account_id(
                &self,
                account_id: &str,
            ) -> YnabResult<Vec<TransactionDetail>>;

            async fn get_transactions_by_account_id_delta(
                &self,
                account_id: &str,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<TransationsDetailDelta>;

            async fn get_transactions_by_account_id_since(
                &self,
                account_id: &str,
                since_date: &str,
            ) -> YnabResult<Vec<TransactionDetail>>;

            async fn get_transactions_by_account_id_since_delta(
                &self,
                account_id: &str,
                since_date: &str,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<TransationsDetailDelta>;

            async fn get_transactions_by_account_id_of(
                &self,
                account_id: &str,
                transaction_type: TransactionType,
            ) -> YnabResult<Vec<TransactionDetail>>;

            async fn get_transactions_by_account_id_of_delta(
                &self,
                account_id: &str,
                transaction_type: TransactionType,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<TransationsDetailDelta>;

            async fn get_transactions_by_account_id_since_date_of(
                &self,
                account_id: &str,
                since_date: &str,
                transaction_type: TransactionType,
            ) -> YnabResult<Vec<TransactionDetail>>;

            async fn get_transactions_by_account_id_since_date_of_delta(
                &self,
                account_id: &str,
                since_date: &str,
                transaction_type: TransactionType,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<TransationsDetailDelta>;

            async fn create_transaction(&self, data: SaveTransaction) -> YnabResult<TransactionDetail>;

            async fn create_transactions(
                &self,
                data: Vec<SaveTransaction>,
            ) -> YnabResult<Vec<TransactionDetail>>;

            async fn update_transactions(
                &self,
                data: Vec<UpdateTransaction>,
            ) -> YnabResult<Vec<TransactionDetail>>;

            async fn import_transactions(&self) -> YnabResult<Vec<String>>;

            async fn get_transaction_by_id(&self, transaction_id: &str) -> YnabResult<TransactionDetail>;

            async fn update_transaction(
                &self,
                transaction_id: &str,
                data: SaveTransaction,
            ) -> YnabResult<TransactionDetail>;
        }
    }
    let ynab_client = Arc::new(MockYnabClient::new());

    let transaction_service = TransactionService::new_boxed(
        ynab_transaction_repo.clone(),
        ynab_transaction_meta_repo.clone(),
        ynab_client.clone(),
    );
    (
        SavingRateService::new_boxed(saving_rate_repo.clone(), transaction_service),
        saving_rate_repo,
        ynab_transaction_repo,
        ynab_transaction_meta_repo,
        ynab_client,
    )
}
