use std::collections::HashSet;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Datelike, NaiveDate};
use datamize_domain::{FinancialResourceMonthly, Month, MonthNum};
use db_sqlite::balance_sheet::sabotage_months_table;
use fake::{faker::chrono::en::Date, Fake};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::months::testutils::TestContext;

async fn check_get_all_from_year(
    pool: SqlitePool,
    year: Option<i32>,
    expected_status: StatusCode,
    expected_resp: Option<Vec<Month>>,
) {
    let context = TestContext::setup(pool);

    if let Some(year) = year {
        context.insert_year(year).await;
    }
    let year = year.unwrap_or(Date().fake::<NaiveDate>().year());

    let expected_resp: Option<Vec<Month>> = expected_resp.map(|resp| {
        let mut seen: HashSet<(MonthNum, i32)> = HashSet::new();
        let mut months: Vec<Month> = resp
            .into_iter()
            .map(|m| Month {
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
            })
            // Filer any month accidently created in double by Dummy data.
            .filter(|m| seen.insert((m.month, m.year)))
            .collect();

        // Empty resources of first month, it should not be in final response.
        if let Some(m) = months.first_mut() {
            m.resources = vec![];
        }

        months
    });

    if let Some(expected_resp) = &expected_resp {
        for m in expected_resp {
            context.set_month(m, m.year).await;
        }
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri(format!("/years/{:?}/months", year))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(mut expected) = expected_resp {
        // Remove months with empty resources as it should not be present in the body of response.
        expected.retain(|e| !e.resources.is_empty());
        // Answer should be sorted by months
        expected.sort_by(|a, b| a.month.cmp(&b.month));
        // Then sort resources by name
        for e in &mut expected {
            e.resources.sort_by(|a, b| a.base.name.cmp(&b.base.name));
        }

        let body: Vec<Month> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_list_when_no_year(pool: SqlitePool) {
    check_get_all_from_year(pool, None, StatusCode::OK, Some(vec![])).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_that_is_in_db(pool: SqlitePool) {
    check_get_all_from_year(
        pool,
        Some(Date().fake::<NaiveDate>().year()),
        StatusCode::OK,
        Some(fake::vec![Month; 3..6]),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_months_table(&pool).await.unwrap();

    check_get_all_from_year(
        pool,
        Some(Date().fake::<NaiveDate>().year()),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
    )
    .await;
}
