use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{FinancialResourceMonthly, Month};
use db_sqlite::balance_sheet::sabotage_months_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::months::testutils::TestContext;

async fn check_get(
    pool: SqlitePool,
    create_year: bool,
    expected_status: StatusCode,
    expected_resp: Option<Month>,
) {
    let context = TestContext::setup(pool);

    let year = expected_resp.clone().unwrap_or_else(|| Faker.fake()).year;
    if create_year {
        context.insert_year(year).await;
    }

    let expected_resp = expected_resp.map(|expected| Month {
        resources: expected
            .resources
            .into_iter()
            .map(|r| FinancialResourceMonthly {
                month: expected.month,
                year: expected.year,
                ..r
            })
            .collect(),
        ..expected
    });

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_month(&expected_resp, year).await;
    }
    let month: i16 = expected_resp
        .clone()
        .unwrap_or_else(|| Faker.fake())
        .month
        .into();

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri(format!("/years/{:?}/months/{:?}", year, month))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(mut expected) = expected_resp {
        // Sort resources by name
        expected
            .resources
            .sort_by(|a, b| a.base.name.cmp(&b.base.name));
        let body: Month = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_no_year(pool: SqlitePool) {
    check_get(pool, false, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_get(pool, true, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    check_get(pool, true, StatusCode::OK, Some(Faker.fake())).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_months_table(&pool).await.unwrap();

    check_get(pool, true, StatusCode::INTERNAL_SERVER_ERROR, None).await;
}
