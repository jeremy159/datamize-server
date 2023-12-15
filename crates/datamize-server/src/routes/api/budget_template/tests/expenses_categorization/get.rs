use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::ExpenseCategorization;
use db_sqlite::budget_template::sabotage_expenses_categorization_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::budget_template::tests::expenses_categorization::testutils::TestContext;

async fn check_get(
    pool: SqlitePool,
    expected_status: StatusCode,
    expected_resp: Option<ExpenseCategorization>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_expenses_categorization(&[expected_resp]).await;
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/expense_categorization/{:?}",
                    expected_resp.clone().unwrap_or_else(|| Faker.fake()).id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = expected_resp {
        let body: ExpenseCategorization = serde_json::from_slice(&body).unwrap();
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
    sabotage_expenses_categorization_table(&pool).await.unwrap();

    check_get(pool, StatusCode::INTERNAL_SERVER_ERROR, None).await;
}
