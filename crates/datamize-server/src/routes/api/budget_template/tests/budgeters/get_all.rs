use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::BudgeterConfig;
use db_sqlite::budget_template::sabotage_budgeters_config_table;
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::budget_template::tests::budgeters::testutils::TestContext;

async fn check_get_all(
    pool: SqlitePool,
    expected_status: StatusCode,
    expected_resp: Option<Vec<BudgeterConfig>>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = &expected_resp {
        context.set_budgeters(expected_resp).await;
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri("/budgeters")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(expected) = expected_resp {
        let body: Vec<BudgeterConfig> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_list_when_nothing_in_db(pool: SqlitePool) {
    check_get_all(pool, StatusCode::OK, Some(vec![])).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_that_is_in_db(pool: SqlitePool) {
    check_get_all(pool, StatusCode::OK, Some(fake::vec![BudgeterConfig; 1..3])).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_budgeters_config_table(&pool).await.unwrap();

    check_get_all(pool, StatusCode::INTERNAL_SERVER_ERROR, None).await;
}
