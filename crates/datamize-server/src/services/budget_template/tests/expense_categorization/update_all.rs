use datamize_domain::ExpenseCategorization;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::budget_template::tests::expense_categorization::testutils::TestContext;

async fn check_update_all(
    pool: SqlitePool,
    new_expenses_categorization: Vec<ExpenseCategorization>,
    already_in_db: Option<Vec<ExpenseCategorization>>,
) {
    let context = TestContext::setup(pool);

    if let Some(already_in_db) = &already_in_db {
        context.set_expenses_categorization(already_in_db).await;
    }

    let response = context
        .service()
        .update_all_expenses_categorization(new_expenses_categorization.clone())
        .await
        .unwrap();

    // Make sure the update is persisted in db
    let saved = context.get_all_expenses_categorization().await.unwrap();
    assert_eq!(response, saved);
    for req in new_expenses_categorization {
        assert!(saved.iter().any(|s| s.name == req.name));
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_empty_list_even_when_nothing_in_db(pool: SqlitePool) {
    check_update_all(pool, Faker.fake(), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let body = fake::vec![ExpenseCategorization; 2..3];
    let mut already_in_db = fake::vec![ExpenseCategorization; 1..2];
    already_in_db[0] = body[0].clone();

    check_update_all(pool, body, Some(already_in_db)).await;
}
