use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::FinancialResourceYearly;
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::resources::testutils::{
    correctly_stub_resource, TestContext,
};

async fn check_delete(
    pool: SqlitePool,
    create_year: bool,
    expected_status: StatusCode,
    expected_resp: Option<FinancialResourceYearly>,
) {
    let context = TestContext::setup(pool);

    let year = expected_resp.clone().unwrap_or_else(|| Faker.fake()).year;
    if create_year {
        context.insert_year(year).await;

        // Create all months
        for m in expected_resp
            .clone()
            .unwrap_or_else(|| Faker.fake())
            .balance_per_month
            .keys()
        {
            context.insert_month(*m, year).await;
        }
    }

    let expected_resp = correctly_stub_resource(expected_resp, year);
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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = expected_resp {
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);

        // Make sure the deletion removed it from db
        let saved = context.get_res(expected.base.id).await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));

        if !expected.balance_per_month.is_empty() {
            // Updates all months that had balance
            for m in expected.balance_per_month.keys() {
                let saved_month = context.get_month(*m, expected.year).await;
                assert!(saved_month.is_ok());

                let saved_month = saved_month.unwrap();
                // Since net_assets are computed from all resources' type
                assert_ne!(saved_month.net_assets.total, 0);
            }
        }

        // Delete the resource also computed net assets of the year
        let saved_year = context.get_year(expected.year).await;
        assert!(saved_year.is_ok());
        let saved_year = saved_year.unwrap();
        assert_ne!(saved_year.net_assets.total, 0);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_no_year(pool: SqlitePool) {
    check_delete(pool, false, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_delete(pool, true, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_deletion(pool: SqlitePool) {
    check_delete(pool, true, StatusCode::OK, Some(Faker.fake())).await;
}
