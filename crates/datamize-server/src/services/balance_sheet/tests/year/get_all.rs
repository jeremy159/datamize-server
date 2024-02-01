use datamize_domain::Year;
use db_sqlite::balance_sheet::sabotage_years_table;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::balance_sheet::tests::year::testutils::{
    assert_err, correctly_stub_years, transform_expected_years, ErrorType, TestContext,
};

async fn check_get_all(
    pool: SqlitePool,
    expected_resp: Option<Vec<Year>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    let expected_resp = correctly_stub_years(expected_resp);

    if let Some(expected_resp) = &expected_resp {
        for y in expected_resp {
            context.set_year(y).await;
        }
    }

    let response = context.into_service().get_all_years().await;
    let expected_resp = transform_expected_years(expected_resp);

    if let Some(expected_resp) = expected_resp {
        assert_eq!(response.unwrap(), expected_resp);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_list_when_nothing_in_db(pool: SqlitePool) {
    check_get_all(pool, Some(vec![]), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_that_is_in_db(pool: SqlitePool) {
    check_get_all(pool, Some(fake::vec![Year; 3..6]), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_internal_when_db_corrupted(pool: SqlitePool) {
    sabotage_years_table(&pool).await.unwrap();

    check_get_all(pool, None, Some(ErrorType::Internal)).await;
}
