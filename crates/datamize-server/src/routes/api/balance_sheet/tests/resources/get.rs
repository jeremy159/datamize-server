use std::collections::HashSet;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{get_all_months_empty, FinancialResourceYearly, YearlyBalances};
use db_sqlite::balance_sheet::sabotage_resources_table;
use fake::{Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::resources::testutils::TestContext;

async fn check_get(
    pool: SqlitePool,
    expected_status: StatusCode,
    expected_resp: Option<FinancialResourceYearly>,
) {
    let context = TestContext::setup(pool).await;
    let mut checked_years = HashSet::<i32>::new();

    // Create all months and years
    for (year, month) in expected_resp
        .clone()
        .unwrap_or_else(|| Faker.fake())
        .iter_months()
    {
        if !checked_years.contains(&year) {
            checked_years.insert(year);
            context.insert_year(year).await;
        }
        context.insert_month(month, year).await;
    }

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_resources(&[expected_resp]).await;
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/resources/{:?}",
                    expected_resp
                        .clone()
                        .unwrap_or_else(|| Faker.fake())
                        .base
                        .id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(mut expected) = expected_resp {
        let years: Vec<_> = expected.iter_years().collect();
        for year in years {
            match expected.get_balance_for_year(year) {
                Some(current_year_balances) => {
                    if current_year_balances.len() < 12 {
                        expected.insert_balance_for_year(year, get_all_months_empty());
                        for (m, b) in current_year_balances {
                            expected.insert_balance_opt(year, m, b);
                        }
                    }
                }
                None => {
                    expected.insert_balance_for_year(year, get_all_months_empty());
                }
            }
        }
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_for_non_existing_resource(pool: SqlitePool) {
    check_get(pool, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    check_get(pool, StatusCode::OK, Some(Faker.fake())).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_resources_table(&pool).await.unwrap();

    check_get(pool, StatusCode::INTERNAL_SERVER_ERROR, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_invalid_id_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool).await;

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .uri(&format!("/resources/{}", Faker.fake::<u32>()))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
