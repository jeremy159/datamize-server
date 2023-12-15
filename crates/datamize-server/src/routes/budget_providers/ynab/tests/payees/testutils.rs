use std::sync::Arc;

use axum::Router;
use datamize_domain::db::ynab::{MockYnabPayeeMetaRepo, YnabPayeeRepo};
use db_sqlite::budget_providers::ynab::SqliteYnabPayeeRepo;
use sqlx::SqlitePool;
use ynab::{MockPayeeRequestsImpl, Payee, PayeesDelta};

use crate::{
    routes::budget_providers::ynab::get_ynab_payee_routes,
    services::budget_providers::YnabPayeeService,
};

pub(crate) struct TestContext {
    ynab_payee_repo: Box<SqliteYnabPayeeRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool, ynab_payees: PayeesDelta) -> Self {
        let ynab_payee_repo = SqliteYnabPayeeRepo::new_boxed(pool.clone());
        let ynab_account_meta_repo = MockYnabPayeeMetaRepo::new_boxed();
        let mut ynab_client = Arc::new(MockPayeeRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_payees_delta()
            .returning(move |_| Ok(ynab_payees.clone()));

        let ynab_account_service = YnabPayeeService::new_boxed(
            ynab_payee_repo.clone(),
            ynab_account_meta_repo,
            ynab_client,
        );
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
