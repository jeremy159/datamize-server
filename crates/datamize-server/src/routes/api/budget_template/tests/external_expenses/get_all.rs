use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::ExternalExpense;
use db_sqlite::budget_template::sabotage_external_expenses_table;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::budget_template::tests::external_expenses::testutils::TestContext;

async fn check_get_all(
    pool: SqlitePool,
    expected_status: StatusCode,
    expected_resp: Option<Vec<ExternalExpense>>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = &expected_resp {
        context.set_external_expenses(expected_resp).await;
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri("/external_expenses")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = expected_resp {
        let body: Vec<ExternalExpense> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_list_when_nothing_in_db(pool: SqlitePool) {
    check_get_all(pool, StatusCode::OK, Some(vec![])).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_that_is_in_db(pool: SqlitePool) {
    check_get_all(
        pool,
        StatusCode::OK,
        Some(fake::vec![ExternalExpense; 1..3]),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_external_expenses_table(&pool).await.unwrap();

    check_get_all(pool, StatusCode::INTERNAL_SERVER_ERROR, None).await;
}
