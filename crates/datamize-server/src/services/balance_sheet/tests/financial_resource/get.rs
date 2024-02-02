use datamize_domain::FinancialResourceYearly;
use db_sqlite::balance_sheet::sabotage_resources_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::financial_resource::testutils::{correctly_stub_resource, TestContext},
    testutils::{assert_err, ErrorType},
};

async fn check_get(
    pool: SqlitePool,
    create_year: bool,
    expected_resp: Option<FinancialResourceYearly>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    let year = expected_resp.clone().unwrap_or_else(|| Faker.fake()).year;
    if create_year {
        context.insert_year(year).await;

        // Create all months
        for m in expected_resp
            .clone()
            .unwrap_or_else(|| Faker.fake())
            .balance_per_month
            .keys()
        {
            context.insert_month(*m, year).await;
        }
    }

    let expected_resp = correctly_stub_resource(expected_resp, year);
    if let Some(expected_resp) = expected_resp.clone() {
        context.set_resources(&[expected_resp]).await;
    }

    let response = context
        .service()
        .get_fin_res(
            expected_resp
                .clone()
                .unwrap_or_else(|| Faker.fake())
                .base
                .id,
        )
        .await;

    if let Some(expected_resp) = expected_resp {
        assert_eq!(response.unwrap(), expected_resp);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_for_non_existing_resource(pool: SqlitePool) {
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
    sabotage_resources_table(&pool).await.unwrap();

    check_get(pool, true, None, Some(ErrorType::Internal)).await;
}
