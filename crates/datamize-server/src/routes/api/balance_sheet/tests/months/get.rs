use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{Month, MonthNum, Uuid};
use fake::{Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::months::testutils::{
    transform_expected_month, TestContext,
};

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

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(expected) = transform_expected_month(expected_resp) {
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
async fn returns_400_for_invalid_year_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .uri(&format!(
                    "/years/{}/months/{:?}",
                    Faker.fake::<Uuid>(),
                    Faker.fake::<MonthNum>()
                ))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_invalid_month_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .uri(&format!(
                    "/years/{}/months/{:?}",
                    Faker.fake::<i32>(),
                    Faker.fake::<Uuid>()
                ))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
