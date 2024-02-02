use datamize_domain::{BudgeterConfig, SaveBudgeterConfig};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    budget_template::tests::budgeter::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

fn are_equal(a: &BudgeterConfig, b: &BudgeterConfig) {
    assert_eq!(a.name, b.name);
    assert_eq!(a.payee_ids, b.payee_ids);
}

async fn check_create(
    pool: SqlitePool,
    new_budgeter: SaveBudgeterConfig,
    expected_resp: Option<BudgeterConfig>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    let response = context.service().create_budgeter(new_budgeter).await;

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        are_equal(&res_body, &expected_resp);

        let saved = context
            .get_budgeter_by_name(&expected_resp.name)
            .await
            .unwrap();
        are_equal(&res_body, &saved);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_budgeter_config(pool: SqlitePool) {
    let body: SaveBudgeterConfig = Faker.fake();
    check_create(pool, body.clone(), Some(body.into()), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_already_exists_when_saving_rate_already_exists(pool: SqlitePool) {
    let body: SaveBudgeterConfig = Faker.fake();
    {
        let context = TestContext::setup(pool.clone());
        context.set_budgeters(&[body.clone().into()]).await;
    }
    check_create(pool, body, None, Some(ErrorType::AlreadyExist)).await;
}
