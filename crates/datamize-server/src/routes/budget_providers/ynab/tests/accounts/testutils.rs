use std::sync::Arc;

use axum::Router;
use datamize_domain::db::ynab::{MockYnabAccountMetaRepo, YnabAccountRepo};
use db_sqlite::budget_providers::ynab::SqliteYnabAccountRepo;
use sqlx::SqlitePool;
use ynab::{Account, AccountsDelta, MockAccountRequestsImpl};

use crate::{
    routes::budget_providers::ynab::get_ynab_account_routes,
    services::budget_providers::YnabAccountService,
};

pub(crate) struct TestContext {
    ynab_account_repo: Box<SqliteYnabAccountRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool, ynab_accounts: AccountsDelta) -> Self {
        let ynab_account_repo = SqliteYnabAccountRepo::new_boxed(pool.clone());
        let ynab_account_meta_repo = MockYnabAccountMetaRepo::new_boxed();
        let mut ynab_client = Arc::new(MockAccountRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_accounts_delta()
            .returning(move |_| Ok(ynab_accounts.clone()));

        let ynab_account_service = YnabAccountService::new_boxed(
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
