use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Datelike, NaiveDate};
use datamize_domain::{
    testutils::{correctly_stub_resources, transform_expected_resources},
    FinancialResourceYearly, Uuid, YearlyBalances,
};
use db_sqlite::balance_sheet::sabotage_resources_table;
use fake::{faker::chrono::en::Date, Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::resources::testutils::TestContext;

async fn check_get_all_from_year(
    pool: SqlitePool,
    year: Option<i32>,
    expected_status: StatusCode,
    expected_resp: Option<Vec<FinancialResourceYearly>>,
) {
    let context = TestContext::setup(pool);

    if let Some(year) = year {
        context.insert_year(year).await;
    }
    let year = year.unwrap_or(Date().fake::<NaiveDate>().year());

    let expected_resp: Option<Vec<FinancialResourceYearly>> =
        correctly_stub_resources(expected_resp, [year, year]);

    if let Some(expected_resp) = &expected_resp {
        for r in expected_resp {
            for (year, month, _) in r.iter_balances() {
                context.insert_month(month, year).await;
            }
        }
        context.set_resources(expected_resp).await;
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri(format!("/years/{:?}/resources", year))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(expected) = transform_expected_resources(expected_resp) {
        let body: Vec<FinancialResourceYearly> = serde_json::from_slice(&body).unwrap();
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
        Some(fake::vec![FinancialResourceYearly; 3..6]),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_resources_table(&pool).await.unwrap();

    check_get_all_from_year(
        pool,
        Some(Date().fake::<NaiveDate>().year()),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_invalid_year_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .uri(&format!("/years/{}/resources", Faker.fake::<Uuid>()))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
