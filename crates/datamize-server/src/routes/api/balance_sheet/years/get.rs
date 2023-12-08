use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::Year;
use db_sqlite::balance_sheet::sabotage_years_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::years::testutils::{
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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

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
