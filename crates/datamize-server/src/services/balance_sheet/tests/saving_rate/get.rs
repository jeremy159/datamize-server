use datamize_domain::SavingRate;
use db_sqlite::balance_sheet::sabotage_saving_rates_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use ynab::TransactionDetail;

use crate::services::balance_sheet::{
    tests::saving_rate::testutils::{assert_err, ErrorType, TestContext},
    SavingRateServiceExt,
};

async fn check_get(
    pool: SqlitePool,
    create_year: bool,
    expected_resp: Option<SavingRate>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool).await;

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
        .into_service()
        .get_saving_rate(expected_resp.clone().unwrap_or_else(|| Faker.fake()).id)
        .await;

    if let Some(mut expected_resp) = expected_resp {
        expected_resp.compute_totals(&transactions);
        assert_eq!(response.unwrap(), expected_resp);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_no_year(pool: SqlitePool) {
    check_get(pool, false, None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_nothing_in_db(pool: SqlitePool) {
    check_get(pool, true, None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    check_get(pool, true, Some(Faker.fake()), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_internal_when_db_corrupted(pool: SqlitePool) {
    sabotage_saving_rates_table(&pool).await.unwrap();

    check_get(pool, true, None, Some(ErrorType::Internal)).await;
}
