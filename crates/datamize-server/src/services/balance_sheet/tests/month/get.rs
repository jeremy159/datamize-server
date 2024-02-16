use datamize_domain::Month;
use db_sqlite::balance_sheet::sabotage_months_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::month::testutils::{transform_expected_month, TestContext},
    testutils::{assert_err, ErrorType},
};

async fn check_get(
    pool: SqlitePool,
    create_year: bool,
    expected_resp: Option<Month>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    let year = expected_resp.clone().unwrap_or_else(|| Faker.fake()).year;
    if create_year {
        context.insert_year(year).await;
    }

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_month(&expected_resp, year).await;
    }
    let month = expected_resp.clone().unwrap_or_else(|| Faker.fake()).month;

    let response = context.service().get_month(month, year).await;
    let expected_resp = transform_expected_month(expected_resp);

    if let Some(expected_resp) = expected_resp {
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
    sabotage_months_table(&pool).await.unwrap();

    check_get(pool, true, None, Some(ErrorType::Database)).await;
}
