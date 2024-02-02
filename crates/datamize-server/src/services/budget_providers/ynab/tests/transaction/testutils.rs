use std::sync::Arc;

use datamize_domain::db::ynab::{DynYnabTransactionMetaRepo, YnabTransactionRepo};
use db_redis::{budget_providers::ynab::RedisYnabTransactionMetaRepo, get_test_pool};
use db_sqlite::budget_providers::ynab::SqliteYnabTransactionRepo;
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{MockTransactionRequestsImpl, TransactionDetail, TransactionsDetailDelta};

use crate::{
    error::AppError,
    services::budget_providers::{
        DynTransactionService, TransactionService, TransactionServiceExt,
    },
};

pub(crate) enum ErrorType {
    Internal,
}

pub(crate) struct TestContext {
    ynab_transaction_repo: Arc<SqliteYnabTransactionRepo>,
    ynab_transaction_meta_repo: DynYnabTransactionMetaRepo,
    transaction_service: DynTransactionService,
}

impl TestContext {
    pub(crate) async fn setup(
        pool: SqlitePool,
        ynab_transactions: TransactionsDetailDelta,
    ) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let ynab_transaction_repo = SqliteYnabTransactionRepo::new_arced(pool);

        let ynab_transaction_meta_repo = RedisYnabTransactionMetaRepo::new_arced(redis_conn_pool);
        ynab_transaction_meta_repo
            .set_delta(Faker.fake())
            .await
            .unwrap();
        let mut ynab_client = Arc::new(MockTransactionRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_transactions_delta()
            .returning(move |_| Ok(ynab_transactions.clone()));

        let transaction_service = TransactionService::new_arced(
            ynab_transaction_repo.clone(),
            ynab_transaction_meta_repo.clone(),
            ynab_client,
        );

        Self {
            ynab_transaction_repo,
            ynab_transaction_meta_repo,
            transaction_service,
        }
    }

    pub(crate) fn service(&self) -> &dyn TransactionServiceExt {
        self.transaction_service.as_ref()
    }

    pub(crate) fn into_service(self) -> DynTransactionService {
        self.transaction_service
    }

    pub(crate) async fn get_transactions(&self) -> Vec<TransactionDetail> {
        self.ynab_transaction_repo.get_all().await.unwrap()
    }

    pub(crate) async fn set_transactions(&self, transactions: &[TransactionDetail]) {
        self.ynab_transaction_repo
            .update_all(transactions)
            .await
            .unwrap();
    }

    pub(crate) async fn get_delta(&self) -> i64 {
        self.ynab_transaction_meta_repo.get_delta().await.unwrap()
    }
}

pub(crate) fn assert_err(err: AppError, expected_err: Option<ErrorType>) {
    match expected_err {
        Some(ErrorType::Internal) => assert!(matches!(err, AppError::InternalServerError(_))),
        None => {
            // noop
        }
    }
}
