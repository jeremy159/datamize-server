use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::BudgeterConfig;
use fake::{Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::budget_template::tests::budgeters::testutils::TestContext;

async fn check_delete(
    pool: SqlitePool,
    expected_status: StatusCode,
    expected_resp: Option<BudgeterConfig>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_budgeters(&[expected_resp]).await;
    }

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/budgeter/{:?}",
                    expected_resp.clone().unwrap_or_else(|| Faker.fake()).id
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
        let body: BudgeterConfig = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);

        // Make sure the deletion removed it from db
        let saved = context.get_budgeter_by_name(&expected.name).await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));
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
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/budgeter/{}", Faker.fake::<u32>()))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
