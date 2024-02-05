use std::sync::Arc;

use datamize_domain::db::{
    ynab::{
        DynYnabScheduledTransactionMetaRepo, DynYnabScheduledTransactionRepo,
        YnabScheduledTransactionMetaRepo,
    },
    DbResult,
};
use db_redis::{budget_providers::ynab::RedisYnabScheduledTransactionMetaRepo, get_test_pool};
use db_sqlite::budget_providers::ynab::SqliteYnabScheduledTransactionRepo;
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{
    MockScheduledTransactionRequestsImpl, ScheduledTransactionDetail,
    ScheduledTransactionsDetailDelta,
};

use crate::services::budget_providers::ScheduledTransactionService;

pub(crate) struct TestContext {
    ynab_scheduled_transaction_repo: DynYnabScheduledTransactionRepo,
    ynab_scheduled_transaction_meta_repo: DynYnabScheduledTransactionMetaRepo,
    scheduled_transaction_service: ScheduledTransactionService,
}

impl TestContext {
    pub(crate) async fn setup(
        pool: SqlitePool,
        ynab_transactions: ScheduledTransactionsDetailDelta,
    ) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let ynab_scheduled_transaction_repo = SqliteYnabScheduledTransactionRepo::new_arced(pool);

        let ynab_scheduled_transaction_meta_repo =
            RedisYnabScheduledTransactionMetaRepo::new_arced(redis_conn_pool);
        ynab_scheduled_transaction_meta_repo
            .set_delta(Faker.fake())
            .await
            .unwrap();
        let mut ynab_client = Arc::new(MockScheduledTransactionRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_scheduled_transactions_delta()
            .returning(move |_| Ok(ynab_transactions.clone()));

        let scheduled_transaction_service = ScheduledTransactionService {
            ynab_scheduled_transaction_repo: ynab_scheduled_transaction_repo.clone(),
            ynab_scheduled_transaction_meta_repo: ynab_scheduled_transaction_meta_repo.clone(),
            ynab_client,
        };

        Self {
            ynab_scheduled_transaction_repo,
            ynab_scheduled_transaction_meta_repo,
            scheduled_transaction_service,
        }
    }

    pub(crate) fn service(&self) -> &ScheduledTransactionService {
        &self.scheduled_transaction_service
    }

    pub(crate) async fn set_transactions(&self, transactions: &[ScheduledTransactionDetail]) {
        self.ynab_scheduled_transaction_repo
            .update_all(transactions)
            .await
            .unwrap();
    }

    pub(crate) async fn get_delta(&self) -> DbResult<i64> {
        self.ynab_scheduled_transaction_meta_repo.get_delta().await
    }

    pub(crate) async fn get_last_saved(&self) -> String {
        self.ynab_scheduled_transaction_meta_repo
            .get_last_saved()
            .await
            .unwrap()
    }

    pub(crate) async fn set_last_saved(&self, date: String) {
        self.ynab_scheduled_transaction_meta_repo
            .set_last_saved(date)
            .await
            .unwrap()
    }
}
