use std::sync::Arc;

use axum::Router;
use datamize_domain::{
    db::{DbResult, FinResRepo, MonthRepo, YearRepo},
    FinancialResourceMonthly, Month, MonthNum, NetTotal, Uuid, Year,
};
use db_sqlite::balance_sheet::{SqliteFinResRepo, SqliteMonthRepo, SqliteYearRepo};
use sqlx::SqlitePool;

use crate::{routes::api::balance_sheet::get_month_routes, services::balance_sheet::MonthService};

pub(crate) struct TestContext {
    year_repo: Arc<SqliteYearRepo>,
    month_repo: Arc<SqliteMonthRepo>,
    fin_res_repo: Arc<SqliteFinResRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool) -> Self {
        let year_repo = SqliteYearRepo::new_arced(pool.clone());
        let month_repo = SqliteMonthRepo::new_arced(pool.clone());
        let fin_res_repo = SqliteFinResRepo::new_arced(pool.clone());

        let month_service = MonthService::new_arced(month_repo.clone());
        let app = get_month_routes(month_service);
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

    pub(crate) async fn set_month(&self, month: &Month, year: i32) {
        self.month_repo.add(month, year).await.unwrap();
        self.set_resources(&month.resources).await;
    }

    pub(crate) async fn set_resources(&self, fin_res: &[FinancialResourceMonthly]) {
        for res in fin_res {
            self.fin_res_repo.update_monthly(res).await.unwrap();
        }
    }

    pub(crate) async fn get_month(&self, month: MonthNum, year: i32) -> DbResult<Month> {
        self.month_repo.get(month, year).await
    }

    pub(crate) async fn get_net_totals(&self, month_id: Uuid) -> DbResult<Vec<NetTotal>> {
        self.month_repo.get_net_totals(month_id).await
    }
}
