use datamize_domain::ExpenseCategorization;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    budget_template::tests::expense_categorization::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

fn are_equal(a: &ExpenseCategorization, b: &ExpenseCategorization) {
    assert_eq!(a.name, b.name);
    assert_eq!(a.expense_type, b.expense_type);
    assert_eq!(a.sub_expense_type, b.sub_expense_type);
}

async fn check_update(
    pool: SqlitePool,
    new_expense_categorization: ExpenseCategorization,
    expected_resp: Option<ExpenseCategorization>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_expenses_categorization(&[expected_resp]).await;
    }

    let response = context
        .service()
        .update_expense_categorization(new_expense_categorization.clone())
        .await;

    if let Some(expected_resp) = expected_resp {
        assert_eq!(response.unwrap(), expected_resp);
        // Make sure the update is persisted in db
        let saved = context
            .get_expense_categorization(new_expense_categorization.id)
            .await
            .unwrap();
        are_equal(&new_expense_categorization, &saved);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_nothing_in_db(pool: SqlitePool) {
    check_update(pool, Faker.fake(), None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let body: ExpenseCategorization = Faker.fake();
    let expected_resp = ExpenseCategorization { ..body.clone() };

    check_update(pool, body, Some(expected_resp), None).await;
}
