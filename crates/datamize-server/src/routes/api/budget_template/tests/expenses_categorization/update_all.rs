use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::ExpenseCategorization;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
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
