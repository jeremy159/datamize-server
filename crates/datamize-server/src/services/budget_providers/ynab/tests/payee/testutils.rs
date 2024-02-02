use std::sync::Arc;

use datamize_domain::db::ynab::{DynYnabPayeeMetaRepo, YnabPayeeMetaRepo, YnabPayeeRepo};
use db_redis::{budget_providers::ynab::RedisYnabPayeeMetaRepo, get_test_pool};
use db_sqlite::budget_providers::ynab::SqliteYnabPayeeRepo;
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{MockPayeeRequestsImpl, Payee, PayeesDelta};

use crate::{
    error::AppError,
    services::budget_providers::{DynYnabPayeeService, YnabPayeeService, YnabPayeeServiceExt},
};

pub(crate) enum ErrorType {
    Internal,
}

pub(crate) struct TestContext {
    ynab_payee_repo: Arc<SqliteYnabPayeeRepo>,
    ynab_payee_service: DynYnabPayeeService,
    ynab_payee_meta_repo: DynYnabPayeeMetaRepo,
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

        let ynab_payee_service = YnabPayeeService::new_arced(
            ynab_payee_repo.clone(),
            ynab_payee_meta_repo.clone(),
            ynab_client,
        );

        Self {
            ynab_payee_repo,
            ynab_payee_service,
            ynab_payee_meta_repo,
        }
    }

    pub(crate) fn service(&self) -> &dyn YnabPayeeServiceExt {
        self.ynab_payee_service.as_ref()
    }

    pub(crate) async fn set_payees(&self, payees: &[Payee]) {
        self.ynab_payee_repo.update_all(payees).await.unwrap();
    }

    pub(crate) async fn get_delta(&self) -> i64 {
        self.ynab_payee_meta_repo.get_delta().await.unwrap()
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
