use datamize_domain::BudgeterConfig;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::budget_template::tests::budgeter::testutils::{
    assert_err, ErrorType, TestContext,
};

async fn check_delete(
    pool: SqlitePool,
    expected_resp: Option<BudgeterConfig>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_budgeters(&[expected_resp]).await;
    }

    let response = context
        .service()
        .delete_budgeter(expected_resp.clone().unwrap_or_else(|| Faker.fake()).id)
        .await;

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        assert_eq!(res_body, expected_resp);

        // Make sure the deletion removed it from db
        let saved = context.get_budgeter_by_name(&expected_resp.name).await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_nothing_in_db(pool: SqlitePool) {
    check_delete(pool, None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_deletion(pool: SqlitePool) {
    check_delete(pool, Some(Faker.fake()), None).await;
}
