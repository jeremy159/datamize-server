use datamize_domain::ExpenseCategorization;
use db_sqlite::budget_template::sabotage_expenses_categorization_table;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::budget_template::tests::expense_categorization::testutils::{
    assert_err, ErrorType, TestContext,
};

async fn check_get_all(
    pool: SqlitePool,
    expected_resp: Option<Vec<ExpenseCategorization>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = &expected_resp {
        context.set_expenses_categorization(expected_resp).await;
    }

    let response = context
        .into_service()
        .get_all_expenses_categorization()
        .await;

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
    check_get_all(pool, Some(fake::vec![ExpenseCategorization; 1..3]), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_500_when_db_corrupted(pool: SqlitePool) {
    sabotage_expenses_categorization_table(&pool).await.unwrap();

    check_get_all(pool, None, Some(ErrorType::Internal)).await;
}
