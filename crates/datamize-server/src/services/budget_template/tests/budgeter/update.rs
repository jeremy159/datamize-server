use datamize_domain::BudgeterConfig;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::budget_template::tests::budgeter::testutils::{
    assert_err, ErrorType, TestContext,
};

fn are_equal(a: &BudgeterConfig, b: &BudgeterConfig) {
    assert_eq!(a.name, b.name);
    assert_eq!(a.payee_ids, b.payee_ids);
}

async fn check_update(
    pool: SqlitePool,
    new_budgeter: BudgeterConfig,
    expected_resp: Option<BudgeterConfig>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_budgeters(&[expected_resp]).await;
    }

    let response = context
        .service()
        .update_budgeter(new_budgeter.clone())
        .await;
    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        assert_eq!(res_body, expected_resp);

        // Make sure the update is persisted in db
        let saved = context
            .get_budgeter_by_name(&expected_resp.name)
            .await
            .unwrap();
        are_equal(&new_budgeter, &saved);
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
    let body: BudgeterConfig = Faker.fake();
    let expected_resp = BudgeterConfig { ..body.clone() };

    check_update(pool, body, Some(expected_resp), None).await;
}
