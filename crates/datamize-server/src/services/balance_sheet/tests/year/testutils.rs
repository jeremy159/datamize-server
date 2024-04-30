use std::{collections::HashSet, sync::Arc};

use datamize_domain::{
    db::{DbResult, FinResRepo, MonthRepo, SavingRateRepo, YearRepo},
    FinancialResourceMonthly, FinancialResourceYearly, Month, MonthNum, NetTotals, SavingRate,
    Uuid, Year,
};
use db_sqlite::balance_sheet::{
    SqliteFinResRepo, SqliteMonthRepo, SqliteSavingRateRepo, SqliteYearRepo,
};
use sqlx::SqlitePool;

use crate::services::balance_sheet::{DynYearService, YearService, YearServiceExt};

pub(crate) struct TestContext {
    year_repo: Arc<SqliteYearRepo>,
    month_repo: Arc<SqliteMonthRepo>,
    fin_res_repo: Arc<SqliteFinResRepo>,
    saving_rate_repo: Arc<SqliteSavingRateRepo>,
    year_service: DynYearService,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool) -> Self {
        let year_repo = SqliteYearRepo::new_arced(pool.clone());
        let month_repo = SqliteMonthRepo::new_arced(pool.clone());
        let fin_res_repo = SqliteFinResRepo::new_arced(pool.clone());
        let saving_rate_repo = SqliteSavingRateRepo::new_arced(pool.clone());

        let year_service = YearService::new_arced(year_repo.clone(), month_repo.clone());
        Self {
            year_repo,
            month_repo,
            fin_res_repo,
            saving_rate_repo,
            year_service,
        }
    }

    pub(crate) fn service(&self) -> &dyn YearServiceExt {
        self.year_service.as_ref()
    }

    pub(crate) fn into_service(self) -> DynYearService {
        self.year_service
    }

    pub(crate) async fn set_year(&self, year: &Year) -> Uuid {
        self.year_repo
            .add(year)
            .await
            .expect("Failed to insert a year.");
        self.set_months(&year.months).await;

        year.id
    }

    pub(crate) async fn set_months(&self, months: &[Month]) {
        for month in months {
            self.month_repo.add(month, month.year).await.unwrap();
            self.set_resources(&month.resources, month.month, month.year)
                .await;
        }
    }

    pub(crate) async fn set_resources(
        &self,
        fin_res: &[FinancialResourceMonthly],
        month: MonthNum,
        year: i32,
    ) {
        for res in fin_res {
            self.fin_res_repo
                .update_monthly(res, month, year)
                .await
                .unwrap();
        }
    }

    pub(crate) async fn get_year(&self, year: i32) -> DbResult<Year> {
        self.year_repo.get(year).await
    }

    pub(crate) async fn get_net_totals(&self, year_id: Uuid) -> DbResult<NetTotals> {
        self.year_repo.get_net_totals(year_id).await
    }

    pub(crate) async fn get_months(&self, year: i32) -> DbResult<Vec<Month>> {
        self.month_repo.get_months_of_year(year).await
    }

    pub(crate) async fn get_saving_rates(&self, year: i32) -> DbResult<Vec<SavingRate>> {
        self.saving_rate_repo.get_from_year(year).await
    }

    pub(crate) async fn get_resources(&self, year: i32) -> DbResult<Vec<FinancialResourceYearly>> {
        self.fin_res_repo.get_from_year(year).await
    }
}

/// Will make sure the related resources and months have the appropriate date associated to them
/// It will also remove months created in duplicate
pub(crate) fn correctly_stub_years(years: Option<Vec<Year>>) -> Option<Vec<Year>> {
    years.map(|years| {
        let mut seen: HashSet<(MonthNum, i32)> = HashSet::new();
        let years: Vec<Year> = years
            .into_iter()
            .map(|y| Year {
                months: y
                    .months
                    .into_iter()
                    .map(|m| Month { year: y.year, ..m })
                    // Filer any month accidently created in double by Dummy data.
                    .filter(|m| seen.insert((m.month, m.year)))
                    .collect(),

                ..y
            })
            .collect();

        years
    })
}

/// Will make sure the related resources and months have the appropriate date associated to them
/// It will also remove months created in duplicate
pub(crate) fn correctly_stub_year(year: Option<Year>) -> Option<Year> {
    year.map(|year| {
        let mut seen: HashSet<(MonthNum, i32)> = HashSet::new();
        Year {
            months: year
                .months
                .into_iter()
                .map(|m| Month {
                    year: year.year,
                    ..m
                })
                // Filer any month accidently created in double by Dummy data.
                .filter(|m| seen.insert((m.month, m.year)))
                .collect(),
            ..year
        }
    })
}

/// Will transform the expected response. In this case, years should be sorted,
/// then months in each years and finally resources in each months.
/// Will also filter out any months with empty resources, as does the API currently.
pub(crate) fn transform_expected_years(expected: Option<Vec<Year>>) -> Option<Vec<Year>> {
    expected.map(|mut expected| {
        // Answer should be sorted by years
        expected.sort_by(|a, b| a.year.cmp(&b.year));
        for y in &mut expected {
            // Remove months with empty resources as it should not be present in the body of response.
            y.months.retain(|m| !m.resources.is_empty());
            // And years by month internally
            y.months.sort_by(|a, b| a.month.cmp(&b.month));
            // Then sort resources by name
            for m in &mut y.months {
                m.resources.sort_by(|a, b| a.base.name.cmp(&b.base.name));
            }
        }
        expected
    })
}

/// Will transform the expected response. In this case, months in year and resources in each months should be sorted.
/// Will also filter out any months with empty resources, as does the API currently.
pub(crate) fn transform_expected_year(expected: Option<Year>) -> Option<Year> {
    expected.map(|mut expected| {
        // Remove months with empty resources as it should not be present in the body of response.
        expected.months.retain(|m| !m.resources.is_empty());
        // And years by month internally
        expected.months.sort_by(|a, b| a.month.cmp(&b.month));
        // Then sort resources by name
        for m in &mut expected.months {
            m.resources.sort_by(|a, b| a.base.name.cmp(&b.base.name));
        }
        expected
    })
}
