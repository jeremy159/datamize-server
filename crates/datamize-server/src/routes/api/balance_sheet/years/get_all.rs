use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::Year;
use db_sqlite::balance_sheet::sabotage_years_table;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::years::testutils::{
    correctly_stub_years, transform_expected_years, TestContext,
};

async fn check_get_all(
    pool: SqlitePool,
    expected_status: StatusCode,
    expected_resp: Option<Vec<Year>>,
) {
    let context = TestContext::setup(pool);

    let expected_resp = correctly_stub_years(expected_resp);

    if let Some(expected_resp) = &expected_resp {
        for y in expected_resp {
            context.set_year(y).await;
        }
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri("/years")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = transform_expected_years(expected_resp) {
        let body: Vec<Year> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_list_when_nothing_in_db(pool: SqlitePool) {
    check_get_all(pool, StatusCode::OK, Some(vec![])).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_that_is_in_db(pool: SqlitePool) {
    check_get_all(pool, StatusCode::OK, Some(fake::vec![Year; 3..6])).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_years_table(&pool).await.unwrap();

    check_get_all(pool, StatusCode::INTERNAL_SERVER_ERROR, None).await;
}
