use std::sync::Arc;

use datamize_domain::{
    db::{external::EncryptionKeyRepo, DbResult, FinResRepo, MonthData, MonthRepo, YearRepo},
    FinancialResourceYearly, Month, MonthNum, Uuid, Year, YearlyBalances,
};
use db_redis::{budget_providers::external::RedisEncryptionKeyRepo, get_test_pool};
use db_sqlite::{
    balance_sheet::{SqliteFinResRepo, SqliteMonthRepo, SqliteYearRepo},
    budget_providers::external::SqliteExternalAccountRepo,
};
use sqlx::SqlitePool;
use ynab::{Account, MockAccountRequestsImpl};

use crate::services::{
    balance_sheet::{DynRefreshFinResService, RefreshFinResService, RefreshFinResServiceExt},
    budget_providers::ExternalAccountService,
};

pub(crate) struct TestContext {
    year_repo: Arc<SqliteYearRepo>,
    month_repo: Arc<SqliteMonthRepo>,
    fin_res_repo: Arc<SqliteFinResRepo>,
    fin_res_service: DynRefreshFinResService,
}

impl TestContext {
    pub(crate) async fn setup(
        pool: SqlitePool,
        ynab_calls: usize,
        ynab_accounts: Vec<Account>,
    ) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let year_repo = SqliteYearRepo::new_arced(pool.clone());
        let month_repo = SqliteMonthRepo::new_arced(pool.clone());
        let fin_res_repo = SqliteFinResRepo::new_arced(pool.clone());
        let external_account_repo = SqliteExternalAccountRepo::new_arced(pool.clone());
        let encryption_key_repo = RedisEncryptionKeyRepo::new_arced(redis_conn_pool);
        encryption_key_repo.set(&fake::vec![u8; 6]).await.unwrap();
        let external_account_service =
            ExternalAccountService::new_arced(external_account_repo.clone(), encryption_key_repo);
        let mut ynab_client = Arc::new(MockAccountRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_accounts()
            .times(ynab_calls)
            .returning(move || Ok(ynab_accounts.clone()));

        let fin_res_service = RefreshFinResService::new_arced(
            fin_res_repo.clone(),
            month_repo.clone(),
            year_repo.clone(),
            external_account_service,
            ynab_client,
        );
        Self {
            year_repo,
            month_repo,
            fin_res_repo,
            fin_res_service,
        }
    }
    pub(crate) fn service(&self) -> &dyn RefreshFinResServiceExt {
        self.fin_res_service.as_ref()
    }

    pub(crate) async fn insert_year(&self, year: i32) -> Uuid {
        let year = Year::new(year);
        self.year_repo
            .add(&year)
            .await
            .expect("Failed to insert a year.");

        year.id
    }

    pub(crate) async fn insert_month(&self, month: MonthNum, year: i32) -> Uuid {
        let month = Month::new(month, year);
        let _ = self.month_repo.add(&month, year).await;

        month.id
    }

    pub(crate) async fn get_month_data(&self, month: MonthNum, year: i32) -> DbResult<MonthData> {
        self.month_repo.get_month_data_by_number(month, year).await
    }

    pub(crate) async fn get_month(&self, month: MonthNum, year: i32) -> DbResult<Month> {
        self.month_repo.get(month, year).await
    }

    pub(crate) async fn set_resources(&self, fin_res: &[FinancialResourceYearly]) {
        for res in fin_res {
            self.fin_res_repo.update(res).await.unwrap();
        }
    }
}

/// Will make sure the resources have the appropriate date associated to them
pub(crate) fn correctly_stub_resources(
    resources: Vec<FinancialResourceYearly>,
    year: i32,
) -> Vec<FinancialResourceYearly> {
    resources
        .into_iter()
        .map(|r| {
            let mut res = FinancialResourceYearly::new(
                r.base.id,
                r.base.name.clone(),
                r.base.resource_type.clone(),
                r.base.ynab_account_ids.clone(),
                r.base.external_account_ids.clone(),
            );
            for (_, month, balance) in r.iter_balances() {
                res.insert_balance(year, month, balance);
            }

            res
        })
        .collect()
}
