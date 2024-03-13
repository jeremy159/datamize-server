use std::sync::Arc;

use axum::Router;
use datamize_domain::db::ynab::{YnabPayeeMetaRepo, YnabPayeeRepo};
use db_redis::{budget_providers::ynab::RedisYnabPayeeMetaRepo, get_test_pool};
use db_sqlite::budget_providers::ynab::SqliteYnabPayeeRepo;
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{MockPayeeRequestsImpl, Payee, PayeesDelta};

use crate::{
    routes::api::budget_providers::ynab::get_ynab_payee_routes,
    services::budget_providers::YnabPayeeService,
};

pub(crate) struct TestContext {
    ynab_payee_repo: Arc<SqliteYnabPayeeRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) async fn setup(pool: SqlitePool, ynab_payees: PayeesDelta) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let ynab_payee_repo = SqliteYnabPayeeRepo::new_arced(pool.clone());
        let ynab_payee_meta_repo = RedisYnabPayeeMetaRepo::new_arced(redis_conn_pool);
        ynab_payee_meta_repo.set_delta(Faker.fake()).await.unwrap();
        let mut ynab_client = Arc::new(MockPayeeRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_payees_delta()
            .returning(move |_| Ok(ynab_payees.clone()));

        let ynab_account_service =
            YnabPayeeService::new_arced(ynab_payee_repo.clone(), ynab_payee_meta_repo, ynab_client);
        let app = get_ynab_payee_routes(ynab_account_service);
        Self {
            ynab_payee_repo,
            app,
        }
    }

    pub(crate) fn into_app(self) -> Router {
        self.app
    }

    pub(crate) async fn set_payees(&self, payees: &[Payee]) {
        self.ynab_payee_repo.update_all(payees).await.unwrap();
    }
}
