use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::FinancialResourceYearly;
use db_sqlite::balance_sheet::sabotage_resources_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::resources::testutils::{
    correctly_stub_resource, TestContext,
};

async fn check_get(
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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = expected_resp {
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_for_non_existing_resource(pool: SqlitePool) {
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
    sabotage_resources_table(&pool).await.unwrap();

    check_get(pool, true, StatusCode::INTERNAL_SERVER_ERROR, None).await;
}
