use datamize_domain::Year;
use db_sqlite::balance_sheet::sabotage_years_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::balance_sheet::tests::year::testutils::{
    assert_err, correctly_stub_year, transform_expected_year, ErrorType, TestContext,
};

async fn check_get(pool: SqlitePool, expected_resp: Option<Year>, expected_err: Option<ErrorType>) {
    let context = TestContext::setup(pool);

    let expected_resp = correctly_stub_year(expected_resp);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_year(&expected_resp).await;
    }

    let response = context
        .into_service()
        .get_year(expected_resp.clone().unwrap_or_else(|| Faker.fake()).year)
        .await;
    let expected_resp = transform_expected_year(expected_resp);

    if let Some(expected_resp) = expected_resp {
        assert_eq!(response.unwrap(), expected_resp);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_nothing_in_db(pool: SqlitePool) {
    check_get(pool, None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    check_get(pool, Some(Faker.fake()), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_internal_when_db_corrupted(pool: SqlitePool) {
    sabotage_years_table(&pool).await.unwrap();

    check_get(pool, None, Some(ErrorType::Internal)).await;
}
