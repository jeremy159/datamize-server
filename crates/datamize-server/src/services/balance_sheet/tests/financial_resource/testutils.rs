use std::{cmp::Ordering, sync::Arc};

use datamize_domain::{
    db::{DbResult, FinResRepo, MonthRepo, YearRepo},
    FinancialResourceYearly, Month, MonthNum, Uuid, Year,
};
use db_sqlite::balance_sheet::{SqliteFinResRepo, SqliteMonthRepo, SqliteYearRepo};
use rand::seq::SliceRandom;
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
    ) -> DbResult<Vec<FinancialResourceYearly>> {
        self.fin_res_repo.get_by_name(res_name).await
    }

    pub(crate) async fn get_month(&self, month: MonthNum, year: i32) -> DbResult<Month> {
        self.month_repo.get(month, year).await
    }

    pub(crate) async fn get_year(&self, year: i32) -> DbResult<Year> {
        self.year_repo.get(year).await
    }
}

/// Will make sure the resources have the appropriate date associated to them
pub(crate) fn correctly_stub_resources(
    resources: Option<Vec<FinancialResourceYearly>>,
    years: [i32; 2],
) -> Option<Vec<FinancialResourceYearly>> {
    resources.map(|resources| {
        resources
            .into_iter()
            .map(|r| {
                let year = *years.choose(&mut rand::thread_rng()).unwrap();
                FinancialResourceYearly { year, ..r }
            })
            .collect()
    })
}

/// Will make sure the resource has the appropriate date associated to it
pub(crate) fn correctly_stub_resource(
    resource: Option<FinancialResourceYearly>,
    year: i32,
) -> Option<FinancialResourceYearly> {
    resource.map(|resource| FinancialResourceYearly { year, ..resource })
}

/// Will transform the expected response. In this case, resources should be sorted by year and then by name.
/// Will also filter out resources that don't have any balance in any month
pub(crate) fn transform_expected_resources(
    expected: Option<Vec<FinancialResourceYearly>>,
) -> Option<Vec<FinancialResourceYearly>> {
    expected.map(|mut expected| {
        // Filter resources without any balances
        expected.retain(|r| !r.balance_per_month.is_empty());
        // Answer should be sorted by years and then by names
        expected.sort_by(|a, b| match a.year.cmp(&b.year) {
            Ordering::Equal => a.base.name.cmp(&b.base.name),
            other => other,
        });
        expected
    })
}
