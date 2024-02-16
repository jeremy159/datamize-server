use datamize_domain::ExpenseCategorization;
use db_sqlite::budget_template::sabotage_expenses_categorization_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    budget_template::tests::expense_categorization::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

async fn check_get(
    pool: SqlitePool,
    expected_resp: Option<ExpenseCategorization>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_expenses_categorization(&[expected_resp]).await;
    }

    let response = context
        .into_service()
        .get_expense_categorization(expected_resp.clone().unwrap_or_else(|| Faker.fake()).id)
        .await;

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
    sabotage_expenses_categorization_table(&pool).await.unwrap();

    check_get(pool, None, Some(ErrorType::Database)).await;
}
