use std::sync::Arc;

use datamize_domain::db::ynab::{YnabCategoryRepo, YnabScheduledTransactionMetaRepo};
use db_redis::{budget_providers::ynab::RedisYnabScheduledTransactionMetaRepo, get_test_pool};
use db_sqlite::budget_providers::ynab::{
    SqliteYnabCategoryRepo, SqliteYnabScheduledTransactionRepo,
};
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{
    Category, MockCategoryRequestsImpl, MockScheduledTransactionRequestsImpl,
    ScheduledTransactionsDetailDelta,
};

use crate::services::{
    budget_providers::ScheduledTransactionService, budget_template::TemplateTransactionService,
};

pub(crate) struct TestContext {
    ynab_category_repo: Arc<SqliteYnabCategoryRepo>,
    template_transaction_service: TemplateTransactionService,
}

impl TestContext {
    pub(crate) async fn setup(
        pool: SqlitePool,
        ynab_scheduled_transactions: ScheduledTransactionsDetailDelta,
        ynab_calls: usize,
    ) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let ynab_category_repo = SqliteYnabCategoryRepo::new_arced(pool.clone());
        let ynab_scheduled_transaction_repo =
            SqliteYnabScheduledTransactionRepo::new_arced(pool.clone());
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
            .returning(move |_| Ok(ynab_scheduled_transactions.clone()));
        let scheduled_transaction_service = ScheduledTransactionService::new_arced(
            ynab_scheduled_transaction_repo,
            ynab_scheduled_transaction_meta_repo,
            ynab_client,
        );

        let mut ynab_client = Arc::new(MockCategoryRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_category_by_id()
            .times(ynab_calls)
            .returning(move |_| Ok(Faker.fake()));

        let template_transaction_service = TemplateTransactionService {
            scheduled_transaction_service,
            ynab_category_repo: ynab_category_repo.clone(),
            ynab_client,
        };

        Self {
            ynab_category_repo,
            template_transaction_service,
        }
    }

    pub(crate) fn into_service(self) -> TemplateTransactionService {
        self.template_transaction_service
    }

    pub(crate) async fn set_categories(&self, categories: &[Category]) {
        self.ynab_category_repo
            .update_all(categories)
            .await
            .unwrap();
    }
}

pub(crate) fn count_sub_transaction_ids(
    ynab_scheduled_transactions: &ScheduledTransactionsDetailDelta,
) -> usize {
    ynab_scheduled_transactions
        .scheduled_transactions
        .iter()
        .filter(|st| !st.deleted)
        .flat_map(|st| &st.subtransactions)
        // .filter(|sub| !sub.deleted) // TODO: Maybe we should filter them in the code?
        .filter_map(|sub| sub.category_id)
        .count()
}
