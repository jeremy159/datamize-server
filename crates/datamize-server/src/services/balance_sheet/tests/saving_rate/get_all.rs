use chrono::{Datelike, NaiveDate};
use datamize_domain::SavingRate;
use db_sqlite::balance_sheet::sabotage_saving_rates_table;
use fake::{faker::chrono::en::Date, Fake};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use ynab::TransactionDetail;

use crate::services::{
    balance_sheet::{tests::saving_rate::testutils::TestContext, SavingRateServiceExt},
    testutils::{assert_err, ErrorType},
};

async fn check_get_all(
    pool: SqlitePool,
    year: Option<i32>,
    expected_resp: Option<Vec<SavingRate>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool).await;

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

    let response = context.into_service().get_all_from_year(year).await;

    if let Some(mut expected_resp) = expected_resp {
        expected_resp
            .iter_mut()
            .for_each(|resp| resp.compute_totals(&transactions));
        assert_eq!(response.unwrap(), expected_resp);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_list_when_no_year(pool: SqlitePool) {
    check_get_all(pool, None, Some(vec![]), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_that_is_in_db(pool: SqlitePool) {
    check_get_all(
        pool,
        Some(Date().fake::<NaiveDate>().year()),
        Some(fake::vec![SavingRate; 1..3]),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_internal_when_db_corrupted(pool: SqlitePool) {
    sabotage_saving_rates_table(&pool).await.unwrap();

    check_get_all(
        pool,
        Some(Date().fake::<NaiveDate>().year()),
        None,
        Some(ErrorType::Internal),
    )
    .await;
}
