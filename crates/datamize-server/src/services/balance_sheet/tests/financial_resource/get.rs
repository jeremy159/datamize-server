use std::collections::HashSet;

use chrono::{Datelike, NaiveDate};
use datamize_domain::{FinancialResourceYearly, YearlyBalances};
use db_sqlite::balance_sheet::sabotage_resources_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::financial_resource::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

async fn check_get(
    pool: SqlitePool,
    expected_resp: Option<FinancialResourceYearly>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);
    let mut checked_years = HashSet::<i32>::new();

    // Create all months and years
    for (year, month) in expected_resp
        .clone()
        .unwrap_or_else(|| Faker.fake())
        .iter_months()
    {
        if !checked_years.contains(&year) {
            checked_years.insert(year);
            context.insert_year(year).await;
        }
        context.insert_month(month, year).await;
    }

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
    check_get(pool, None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    let mut res: FinancialResourceYearly = Faker.fake();
    res.clear_all_balances();
    let current_date = Faker.fake::<NaiveDate>();
    let month = current_date.month().try_into().unwrap();
    let year = current_date.year();
    res.insert_balance(year, month, (-1000000..1000000).fake());

    check_get(pool, Some(res), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_internal_when_db_corrupted(pool: SqlitePool) {
    sabotage_resources_table(&pool).await.unwrap();

    check_get(pool, None, Some(ErrorType::Database)).await;
}
