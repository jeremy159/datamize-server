use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Datelike, NaiveDate};
use datamize_domain::SavingRate;
use db_sqlite::balance_sheet::sabotage_saving_rates_table;
use fake::{faker::chrono::en::Date, Fake};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;
use ynab::TransactionDetail;

use crate::routes::api::balance_sheet::tests::saving_rates::testutils::TestContext;

async fn check_get_all(
    pool: SqlitePool,
    year: Option<i32>,
    expected_status: StatusCode,
    expected_resp: Option<Vec<SavingRate>>,
) {
    let context = TestContext::setup(pool);

    if let Some(year) = year {
        context.insert_year(year).await;
    }
    let year = year.unwrap_or(Date().fake::<NaiveDate>().year());

    let expected_resp: Option<Vec<SavingRate>> =
        expected_resp.map(|resp| resp.into_iter().map(|s| SavingRate { year, ..s }).collect());

    if let Some(expected_resp) = &expected_resp {
        context.set_saving_rates(expected_resp).await;
    }

    let transactions = fake::vec![TransactionDetail; 1..5];
    context.set_transactions(&transactions).await;

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri(format!("/years/{:?}/saving_rates", year)) // TODO: Test when passing wrong format (e.g. a uuid instead of year number), most probably in the integration tests.
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(mut expected_resp) = expected_resp {
        expected_resp
            .iter_mut()
            .for_each(|resp| resp.compute_totals(&transactions));
        let body: Vec<SavingRate> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected_resp);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_list_when_no_year(pool: SqlitePool) {
    check_get_all(pool, None, StatusCode::OK, Some(vec![])).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_that_is_in_db(pool: SqlitePool) {
    check_get_all(
        pool,
        Some(Date().fake::<NaiveDate>().year()),
        StatusCode::OK,
        Some(fake::vec![SavingRate; 1..3]),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_saving_rates_table(&pool).await.unwrap();

    check_get_all(
        pool,
        Some(Date().fake::<NaiveDate>().year()),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
    )
    .await;
}
