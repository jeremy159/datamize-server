use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{ExpenseCategorization, Uuid};
use fake::{Dummy, Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serde::Serialize;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::budget_template::tests::expenses_categorization::testutils::TestContext;

async fn check_update_all(
    pool: SqlitePool,
    req_body: Option<Vec<ExpenseCategorization>>,
    expected_status: StatusCode,
    already_in_db: Option<Vec<ExpenseCategorization>>,
) {
    let context = TestContext::setup(pool);

    if let Some(already_in_db) = &already_in_db {
        context.set_expenses_categorization(already_in_db).await;
    }

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/expenses_categorization")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Vec<ExpenseCategorization> = serde_json::from_slice(&body).unwrap();
    if let Some(req_body) = req_body {
        // Make sure the update is persisted in db
        let saved = context.get_all_expenses_categorization().await.unwrap();
        assert_eq!(body, saved);
        for req in &req_body {
            assert!(saved.iter().any(|s| s.name == req.name));
        }
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_200_even_when_nothing_in_db(pool: SqlitePool) {
    check_update_all(pool, Some(Faker.fake()), StatusCode::OK, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let body = fake::vec![ExpenseCategorization; 2..3];
    let mut already_in_db = fake::vec![ExpenseCategorization; 1..2];
    already_in_db[0] = body[0].clone();

    check_update_all(pool, Some(body), StatusCode::OK, Some(already_in_db)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_422_for_invalid_body_format_data(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    #[derive(Debug, Clone, Serialize, Dummy)]
    struct ReqBody {
        pub id: Uuid,
        pub name: String,
    }
    let body = Faker.fake::<ReqBody>();

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/expenses_categorization")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_empty_body(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/expenses_categorization")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_415_for_missing_json_content_type(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let body = Faker.fake::<Vec<ExpenseCategorization>>();

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/expenses_categorization")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
