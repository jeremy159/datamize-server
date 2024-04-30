use std::sync::Arc;

use axum::Router;
use datamize_domain::{
    db::{DbResult, FinResRepo, MonthRepo, YearRepo},
    FinancialResourceYearly, Month, MonthNum, Uuid, Year,
};
use db_redis::{balance_sheet::resource::RedisFinResOrderRepo, get_test_pool};
use db_sqlite::balance_sheet::{SqliteFinResRepo, SqliteMonthRepo, SqliteYearRepo};
use sqlx::SqlitePool;

use crate::{
    routes::api::balance_sheet::get_fin_res_routes, services::balance_sheet::FinResService,
};

pub(crate) struct TestContext {
    year_repo: Arc<SqliteYearRepo>,
    month_repo: Arc<SqliteMonthRepo>,
    fin_res_repo: Arc<SqliteFinResRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) async fn setup(pool: SqlitePool) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let year_repo = SqliteYearRepo::new_arced(pool.clone());
        let month_repo = SqliteMonthRepo::new_arced(pool.clone());
        let fin_res_repo = SqliteFinResRepo::new_arced(pool.clone());
        let fin_res_order_repo = RedisFinResOrderRepo::new_arced(redis_conn_pool);

        let fin_res_service = FinResService::new_arced(
            fin_res_repo.clone(),
            month_repo.clone(),
            year_repo.clone(),
            fin_res_order_repo,
        );
        let app = get_fin_res_routes(fin_res_service);
        Self {
            year_repo,
            month_repo,
            fin_res_repo,
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
