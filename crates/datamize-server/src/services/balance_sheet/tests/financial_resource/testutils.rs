use std::sync::Arc;

use datamize_domain::{
    db::{DbResult, FinResRepo, MonthRepo, YearRepo},
    FinancialResourceYearly, Month, MonthNum, Uuid, Year,
};
use db_sqlite::balance_sheet::{SqliteFinResRepo, SqliteMonthRepo, SqliteYearRepo};
use sqlx::SqlitePool;

use crate::services::balance_sheet::{DynFinResService, FinResService, FinResServiceExt};

pub(crate) struct TestContext {
    year_repo: Arc<SqliteYearRepo>,
    month_repo: Arc<SqliteMonthRepo>,
    fin_res_repo: Arc<SqliteFinResRepo>,
    fin_res_service: DynFinResService,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool) -> Self {
        let year_repo = SqliteYearRepo::new_arced(pool.clone());
        let month_repo = SqliteMonthRepo::new_arced(pool.clone());
        let fin_res_repo = SqliteFinResRepo::new_arced(pool.clone());

        let fin_res_service =
            FinResService::new_arced(fin_res_repo.clone(), month_repo.clone(), year_repo.clone());
        Self {
            year_repo,
            month_repo,
            fin_res_repo,
            fin_res_service,
        }
    }

    pub(crate) fn service(&self) -> &dyn FinResServiceExt {
        self.fin_res_service.as_ref()
    }

    pub(crate) fn into_service(self) -> DynFinResService {
        self.fin_res_service
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

    pub(crate) async fn set_resource(&self, resource: &FinancialResourceYearly) {
        self.fin_res_repo.update(resource).await.unwrap();
    }

    pub(crate) async fn set_resources(&self, fin_res: &[FinancialResourceYearly]) {
        for res in fin_res {
            self.fin_res_repo.update(res).await.unwrap();
        }
    }

    pub(crate) async fn get_res(&self, res_id: Uuid) -> DbResult<FinancialResourceYearly> {
        self.fin_res_repo.get(res_id).await
    }

    pub(crate) async fn get_res_by_name(
        &self,
        res_name: &str,
    ) -> DbResult<FinancialResourceYearly> {
        self.fin_res_repo.get_by_name(res_name).await
    }

    pub(crate) async fn get_month(&self, month: MonthNum, year: i32) -> DbResult<Month> {
        self.month_repo.get(month, year).await
    }

    pub(crate) async fn get_years(&self) -> DbResult<Vec<Year>> {
        self.year_repo.get_years().await
    }
}
