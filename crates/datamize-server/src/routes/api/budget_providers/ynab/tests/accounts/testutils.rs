use std::sync::Arc;

use axum::Router;
use datamize_domain::db::ynab::{YnabAccountMetaRepo, YnabAccountRepo};
use db_redis::{budget_providers::ynab::RedisYnabAccountMetaRepo, get_test_pool};
use db_sqlite::budget_providers::ynab::SqliteYnabAccountRepo;
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{Account, AccountsDelta, MockAccountRequestsImpl};

use crate::{
    routes::api::budget_providers::ynab::get_ynab_account_routes,
    services::budget_providers::YnabAccountService,
};

pub(crate) struct TestContext {
    ynab_account_repo: Arc<SqliteYnabAccountRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) async fn setup(pool: SqlitePool, ynab_accounts: AccountsDelta) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let ynab_account_repo = SqliteYnabAccountRepo::new_arced(pool.clone());
        let ynab_account_meta_repo = RedisYnabAccountMetaRepo::new_arced(redis_conn_pool);
        ynab_account_meta_repo
            .set_delta(Faker.fake())
            .await
            .unwrap();
        let mut ynab_client = Arc::new(MockAccountRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_accounts_delta()
            .returning(move |_| Ok(ynab_accounts.clone()));

        let ynab_account_service = YnabAccountService::new_arced(
            ynab_account_repo.clone(),
            ynab_account_meta_repo,
            ynab_client,
        );
        let app = get_ynab_account_routes(ynab_account_service);
        Self {
            ynab_account_repo,
            app,
        }
    }

    pub(crate) fn into_app(self) -> Router {
        self.app
    }

    pub(crate) async fn set_accounts(&self, accounts: &[Account]) {
        self.ynab_account_repo.update_all(accounts).await.unwrap();
    }
}
