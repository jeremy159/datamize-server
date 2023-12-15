use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::ExternalExpense;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::budget_template::tests::external_expenses::testutils::TestContext;

async fn check_delete(
    pool: SqlitePool,
    expected_status: StatusCode,
    expected_resp: Option<ExternalExpense>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_external_expenses(&[expected_resp]).await;
    }

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/external_expense/{:?}",
                    expected_resp.clone().unwrap_or_else(|| Faker.fake()).id
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
        let body: ExternalExpense = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);

        // Make sure the deletion removed it from db
        let saved = context.get_external_expense_by_name(&expected.name).await;
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
