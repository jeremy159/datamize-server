use std::sync::Arc;

use datamize_domain::{
    db::{ynab::YnabTransactionRepo, DbResult, SavingRateRepo, YearRepo},
    SavingRate, Uuid, Year,
};
use db_redis::{budget_providers::ynab::RedisYnabTransactionMetaRepo, get_test_pool};
use db_sqlite::{
    balance_sheet::{SqliteSavingRateRepo, SqliteYearRepo},
    budget_providers::ynab::SqliteYnabTransactionRepo,
};
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{MockTransactionRequestsImpl, TransactionDetail};

use crate::{
    error::AppError,
    services::{balance_sheet::SavingRateService, budget_providers::TransactionService},
};

pub(crate) enum ErrorType {
    Internal,
    NotFound,
    AlreadyExist,
}

pub(crate) struct TestContext {
    year_repo: Arc<SqliteYearRepo>,
    saving_rate_repo: Arc<SqliteSavingRateRepo>,
    ynab_transaction_repo: Arc<SqliteYnabTransactionRepo>,
    saving_rate_service: SavingRateService,
}

impl TestContext {
    pub(crate) async fn setup(pool: SqlitePool) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let year_repo = SqliteYearRepo::new_arced(pool.clone());
        let saving_rate_repo = SqliteSavingRateRepo::new_arced(pool.clone());
        let ynab_transaction_repo = SqliteYnabTransactionRepo::new_arced(pool);

        let ynab_transaction_meta_repo = RedisYnabTransactionMetaRepo::new_arced(redis_conn_pool);

        let mut ynab_client = Arc::new(MockTransactionRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_transactions_delta()
            .returning(|_| Ok(Faker.fake()));

        let transaction_service = TransactionService::new_arced(
            ynab_transaction_repo.clone(),
            ynab_transaction_meta_repo,
            ynab_client,
        );
        let saving_rate_service = SavingRateService {
            saving_rate_repo: saving_rate_repo.clone(),
            transaction_service,
        };

        Self {
            year_repo,
            saving_rate_repo,
            ynab_transaction_repo,
            saving_rate_service,
        }
    }

    pub(crate) fn service(&self) -> &SavingRateService {
        &self.saving_rate_service
    }

    pub(crate) fn into_service(self) -> SavingRateService {
        self.saving_rate_service
    }

    pub(crate) async fn insert_year(&self, year: i32) -> Uuid {
        let year = Year::new(year);
        self.year_repo
            .add(&year)
            .await
            .expect("Failed to insert a year.");

        year.id
    }

    pub(crate) async fn set_saving_rates(&self, saving_rates: &[SavingRate]) {
        for saving_rate in saving_rates {
            self.saving_rate_repo.update(saving_rate).await.unwrap();
        }
    }

    pub(crate) async fn set_transactions(&self, transactions: &[TransactionDetail]) {
        self.ynab_transaction_repo
            .update_all(transactions)
            .await
            .unwrap();
    }

    pub(crate) async fn get_saving_rate_by_name(&self, name: &str) -> DbResult<SavingRate> {
        self.saving_rate_repo.get_by_name(name).await
    }
}

pub(crate) fn assert_err(err: AppError, expected_err: Option<ErrorType>) {
    match expected_err {
        Some(ErrorType::Internal) => assert!(matches!(err, AppError::InternalServerError(_))),
        Some(ErrorType::NotFound) => {
            assert!(matches!(err, AppError::ResourceNotFound))
        }
        Some(ErrorType::AlreadyExist) => assert!(matches!(err, AppError::ResourceAlreadyExist)),
        None => {
            // noop
        }
    }
}
