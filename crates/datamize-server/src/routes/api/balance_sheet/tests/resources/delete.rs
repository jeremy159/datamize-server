use std::collections::HashSet;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{FinancialResourceYearly, YearlyBalances};
use fake::{Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::resources::testutils::TestContext;

async fn check_delete(
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
        .app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/resources/{:?}",
                    expected_resp
                        .clone()
                        .unwrap_or_else(|| Faker.fake())
                        .base
                        .id
                ))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(expected) = expected_resp {
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);

        // Make sure the deletion removed it from db
        let saved = context.get_res(expected.base.id).await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));

        if !expected.is_empty() {
            // Updates all months that had balance
            for (year, month, _) in expected.iter_balances() {
                let saved_month = context.get_month(month, year).await;
                assert!(saved_month.is_ok());

                let saved_month = saved_month.unwrap();
                if !saved_month.resources.is_empty() {
                    // Since net_assets are computed from all resources' type
                    assert_ne!(saved_month.net_assets().total, 0);
                }
            }
        }

        // Delete the resource also computed net assets of the year
        let saved_years = context.get_years().await;
        assert!(saved_years.is_ok());
        let saved_years = saved_years.unwrap();
        for saved_year in saved_years {
            if let Some(last_month) = saved_year.get_last_month() {
                assert_eq!(saved_year.net_assets().total, last_month.net_assets().total);
            }
        }
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_delete(pool, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_deletion(pool: SqlitePool) {
    check_delete(pool, StatusCode::OK, Some(Faker.fake())).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_invalid_id_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool).await;

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/resources/{}", Faker.fake::<u32>()))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
