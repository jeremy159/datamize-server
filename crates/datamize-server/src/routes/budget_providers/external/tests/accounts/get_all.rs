use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::ExternalAccount;
use db_sqlite::budget_providers::external::sabotage_external_accounts_table;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::budget_providers::external::tests::accounts::testutils::{
    correctly_stub_accounts, TestContext,
};

async fn check_get_all(
    pool: SqlitePool,
    expected_status: StatusCode,
    expected_resp: Option<Vec<ExternalAccount>>,
) {
    let context = TestContext::setup(pool).await;

    if let Some(expected_resp) = expected_resp.clone() {
        context
            .set_accounts(&correctly_stub_accounts(expected_resp))
            .await;
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri("/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = expected_resp {
        let body: Vec<ExternalAccount> = serde_json::from_slice(&body).unwrap();
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
        Some(fake::vec![ExternalAccount; 1..3]),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_external_accounts_table(&pool).await.unwrap();

    check_get_all(pool, StatusCode::INTERNAL_SERVER_ERROR, None).await;
}
