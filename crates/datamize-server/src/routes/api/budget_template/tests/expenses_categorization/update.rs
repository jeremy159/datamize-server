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

fn are_equal(a: &ExpenseCategorization, b: &ExpenseCategorization) {
    assert_eq!(a.name, b.name);
    assert_eq!(a.expense_type, b.expense_type);
    assert_eq!(a.sub_expense_type, b.sub_expense_type);
}

async fn check_update(
    pool: SqlitePool,
    req_body: Option<ExpenseCategorization>,
    expected_status: StatusCode,
    expected_resp: Option<ExpenseCategorization>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_expenses_categorization(&[expected_resp]).await;
    }

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/expense_categorization/{:?}",
                    req_body
                        .clone()
                        .expect("missing body to fetch saving rate")
                        .id
                ))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(expected) = expected_resp {
        let body: ExpenseCategorization = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);
        if let Some(req_body) = req_body {
            // Make sure the update is persisted in db
            let saved = context
                .get_expense_categorization(req_body.id)
                .await
                .unwrap();
            are_equal(&req_body, &saved);
        }
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_update(pool, Some(Faker.fake()), StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let body: ExpenseCategorization = Faker.fake();
    let expected_resp = ExpenseCategorization { ..body.clone() };

    check_update(pool, Some(body), StatusCode::OK, Some(expected_resp)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_invalid_id_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/expense_categorization/{}", Faker.fake::<u32>()))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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
                .uri(&format!("/expense_categorization/{}", Faker.fake::<Uuid>()))
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
                .uri(&format!("/expense_categorization/{}", Faker.fake::<Uuid>()))
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

    let body = Faker.fake::<ExpenseCategorization>();

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/expense_categorization/{}", Faker.fake::<Uuid>()))
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
