use std::{cmp::Ordering, collections::HashSet, sync::Arc};

use axum::Router;
use datamize_domain::{
    db::{DbResult, FinResRepo, MonthRepo, YearRepo},
    FinancialResourceMonthly, Month, MonthNum, NetTotal, Uuid, Year,
};
use db_sqlite::balance_sheet::{SqliteFinResRepo, SqliteMonthRepo, SqliteYearRepo};
use rand::seq::SliceRandom;
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

/// Will make sure the related resources and months have the appropriate date associated to them
/// It will also remove months created in duplicate
pub(crate) fn correctly_stub_months(
    months: Option<Vec<Month>>,
    years: [i32; 2],
) -> Option<Vec<Month>> {
    months.map(|months| {
        let mut seen: HashSet<(MonthNum, i32)> = HashSet::new();
        let mut months: Vec<Month> = months
            .into_iter()
            .map(|m| {
                let year = *years.choose(&mut rand::thread_rng()).unwrap();
                Month {
                    year,
                    resources: m
                        .resources
                        .into_iter()
                        .map(|r| FinancialResourceMonthly {
                            year,
                            month: m.month,
                            ..r
                        })
                        .collect(),
                    ..m
                }
            })
            // Filer any month accidently created in double by Dummy data.
            .filter(|m| seen.insert((m.month, m.year)))
            .collect();

        // Empty resources of first month, it should not be in final response.
        if let Some(m) = months.first_mut() {
            m.resources = vec![];
        }

        months
    })
}

/// Will make sure the related resources have the appropriate date associated to them
pub(crate) fn correctly_stub_month(month: Option<Month>) -> Option<Month> {
    month.map(|month| Month {
        resources: month
            .resources
            .into_iter()
            .map(|r| FinancialResourceMonthly {
                month: month.month,
                year: month.year,
                ..r
            })
            .collect(),
        ..month
    })
}

/// Will transform the expected response. In this case, months should be sorted,
/// and resources in each months.
/// Will also filter out any months with empty resources, as does the API currently.
pub(crate) fn transform_expected_months(expected: Option<Vec<Month>>) -> Option<Vec<Month>> {
    expected.map(|mut expected| {
        // Remove months with empty resources as it should not be present in the body of response.
        expected.retain(|e| !e.resources.is_empty());
        // Answer should be sorted by years and then months
        expected.sort_by(|a, b| match a.year.cmp(&b.year) {
            Ordering::Equal => a.month.cmp(&b.month),
            other => other,
        });
        // Then sort resources by name
        for e in &mut expected {
            e.resources.sort_by(|a, b| a.base.name.cmp(&b.base.name));
        }
        expected
    })
}

/// Will transform the expected response. In this case, months should be sorted,
/// and resources in each months.
/// Will also filter out any months with empty resources, as does the API currently.
pub(crate) fn transform_expected_month(expected: Option<Month>) -> Option<Month> {
    expected.map(|mut expected| {
        // Sort resources by name
        expected
            .resources
            .sort_by(|a, b| a.base.name.cmp(&b.base.name));
        expected
    })
}
