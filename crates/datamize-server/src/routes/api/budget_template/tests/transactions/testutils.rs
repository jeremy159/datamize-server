use std::sync::Arc;

use axum::Router;
use datamize_domain::db::ynab::{MockYnabScheduledTransactionMetaRepo, YnabCategoryRepo};
use db_sqlite::budget_providers::ynab::{
    SqliteYnabCategoryRepo, SqliteYnabScheduledTransactionRepo,
};
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{
    Category, MockCategoryRequestsImpl, MockScheduledTransactionRequestsImpl,
    ScheduledTransactionsDetailDelta,
};

use crate::{
    routes::api::budget_template::get_transaction_routes,
    services::budget_template::{ScheduledTransactionService, TemplateTransactionService},
};

pub(crate) struct TestContext {
    ynab_category_repo: Box<SqliteYnabCategoryRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) fn setup(
        pool: SqlitePool,
        ynab_scheduled_transactions: ScheduledTransactionsDetailDelta,
        ynab_calls: usize,
    ) -> Self {
        let ynab_category_repo = SqliteYnabCategoryRepo::new_boxed(pool.clone());
        let ynab_scheduled_transaction_repo =
            SqliteYnabScheduledTransactionRepo::new_boxed(pool.clone());
        let ynab_scheduled_transaction_meta_repo =
            MockYnabScheduledTransactionMetaRepo::new_boxed();
        let mut ynab_client = Arc::new(MockScheduledTransactionRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_scheduled_transactions_delta()
            .returning(move |_| Ok(ynab_scheduled_transactions.clone()));
        let scheduled_transaction_service = ScheduledTransactionService::new_boxed(
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

        let template_transaction_service = TemplateTransactionService::new_boxed(
            scheduled_transaction_service,
            ynab_category_repo.clone(),
            ynab_client,
        );
        let app = get_transaction_routes(template_transaction_service);
        Self {
            ynab_category_repo,
            app,
        }
    }

    pub(crate) fn into_app(self) -> Router {
        self.app
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
