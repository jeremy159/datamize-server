use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::SavingRate;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;
use ynab::TransactionDetail;

use crate::routes::api::balance_sheet::testutils::TestContext;

async fn check_delete(
    pool: SqlitePool,
    create_year: bool,
    expected_status: StatusCode,
    expected_resp: Option<SavingRate>,
) {
    let context = TestContext::setup(pool);

    if create_year {
        let year = expected_resp.clone().unwrap_or_else(|| Faker.fake()).year;
        context.insert_year(year).await;
    }

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_saving_rates(&[expected_resp]).await;
    }

    let transactions = fake::vec![TransactionDetail; 1..5];
    context.set_transactions(&transactions).await;

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/saving_rates/{:?}",
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

    if let Some(mut expected_resp) = expected_resp {
        expected_resp.compute_totals(&transactions);
        let body: SavingRate = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected_resp);

        // Make sure the deletion removed it from db
        let saved = context.get_saving_rate_by_name(&expected_resp.name).await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_no_year(pool: SqlitePool) {
    check_delete(pool, false, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_delete(pool, true, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_deletion(pool: SqlitePool) {
    check_delete(pool, true, StatusCode::OK, Some(Faker.fake())).await;
}
