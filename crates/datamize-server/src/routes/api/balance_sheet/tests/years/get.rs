use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{Uuid, Year};
use db_sqlite::balance_sheet::sabotage_years_table;
use fake::{Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::years::testutils::{
    correctly_stub_year, transform_expected_year, TestContext,
};

async fn check_get(pool: SqlitePool, expected_status: StatusCode, expected_resp: Option<Year>) {
    let context = TestContext::setup(pool);

    let expected_resp = correctly_stub_year(expected_resp);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_year(&expected_resp).await;
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/years/{:?}",
                    expected_resp.clone().unwrap_or_else(|| Faker.fake()).year
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(expected) = transform_expected_year(expected_resp) {
        let body: Year = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_get(pool, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    check_get(pool, StatusCode::OK, Some(Faker.fake())).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_years_table(&pool).await.unwrap();

    check_get(pool, StatusCode::INTERNAL_SERVER_ERROR, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_invalid_year_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .uri(&format!("/years/{}", Faker.fake::<Uuid>()))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
