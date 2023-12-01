use std::sync::Arc;

use axum::Router;
use datamize_domain::{
    db::{
        ynab::{MockYnabTransactionMetaRepo, YnabTransactionRepo},
        DbResult, SavingRateRepo, YearRepo,
    },
    SavingRate, Uuid, Year,
};
use db_sqlite::{
    balance_sheet::{SqliteSavingRateRepo, SqliteYearRepo},
    budget_providers::ynab::SqliteYnabTransactionRepo,
};
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{MockTransactionRequestsImpl, TransactionDetail};

use crate::{
    routes::api::balance_sheet::get_saving_rate_routes,
    services::{balance_sheet::SavingRateService, budget_providers::TransactionService},
};

pub(crate) struct TestContext {
    year_repo: Arc<SqliteYearRepo>,
    saving_rate_repo: Arc<SqliteSavingRateRepo>,
    ynab_transaction_repo: Box<SqliteYnabTransactionRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool) -> Self {
        let year_repo = SqliteYearRepo::new_arced(pool.clone());
        let saving_rate_repo = SqliteSavingRateRepo::new_arced(pool.clone());
        let ynab_transaction_repo = SqliteYnabTransactionRepo::new_boxed(pool);

        let ynab_transaction_meta_repo = MockYnabTransactionMetaRepo::new_boxed();

        let mut ynab_client = Arc::new(MockTransactionRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_transactions_delta()
            .returning(|_| Ok(Faker.fake()));

        let transaction_service = TransactionService::new_boxed(
            ynab_transaction_repo.clone(),
            ynab_transaction_meta_repo,
            ynab_client,
        );
        let saving_rate_service =
            SavingRateService::new_boxed(saving_rate_repo.clone(), transaction_service);
        let app = get_saving_rate_routes(saving_rate_service);
        Self {
            year_repo,
            saving_rate_repo,
            ynab_transaction_repo,
            app,
        }
    }

    pub(crate) fn app(&self) -> Router {
        self.app.clone()
    }

    pub(crate) fn into_app(self) -> Router {
        self.app
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
