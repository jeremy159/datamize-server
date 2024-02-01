use std::sync::Arc;

use datamize_domain::db::ynab::{DynYnabAccountMetaRepo, YnabAccountMetaRepo, YnabAccountRepo};
use db_redis::{budget_providers::ynab::RedisYnabAccountMetaRepo, get_test_pool};
use db_sqlite::budget_providers::ynab::SqliteYnabAccountRepo;
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{Account, AccountsDelta, MockAccountRequestsImpl};

use crate::{
    error::AppError,
    services::budget_providers::{
        DynYnabAccountService, YnabAccountService, YnabAccountServiceExt,
    },
};

pub(crate) enum ErrorType {
    Internal,
}

pub(crate) struct TestContext {
    ynab_account_repo: Arc<SqliteYnabAccountRepo>,
    ynab_account_service: DynYnabAccountService,
    ynab_account_meta_repo: DynYnabAccountMetaRepo,
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
            ynab_account_meta_repo.clone(),
            ynab_client,
        );

        Self {
            ynab_account_repo,
            ynab_account_service,
            ynab_account_meta_repo,
        }
    }

    pub(crate) fn service(&self) -> &dyn YnabAccountServiceExt {
        self.ynab_account_service.as_ref()
    }

    pub(crate) async fn set_accounts(&self, accounts: &[Account]) {
        self.ynab_account_repo.update_all(accounts).await.unwrap();
    }

    pub(crate) async fn get_delta(&self) -> i64 {
        self.ynab_account_meta_repo.get_delta().await.unwrap()
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
