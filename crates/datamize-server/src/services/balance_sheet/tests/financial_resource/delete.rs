use std::collections::HashSet;

use datamize_domain::{get_all_months_empty, FinancialResourceYearly, YearlyBalances};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::financial_resource::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

async fn check_delete(
    pool: SqlitePool,
    expected_resp: Option<FinancialResourceYearly>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool).await;
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
        .delete_fin_res(
            expected_resp
                .clone()
                .unwrap_or_else(|| Faker.fake())
                .base
                .id,
        )
        .await;

    if let Some(mut expected_resp) = expected_resp {
        let years: Vec<_> = expected_resp.iter_years().collect();
        for year in years {
            match expected_resp.get_balance_for_year(year) {
                Some(current_year_balances) => {
                    if current_year_balances.len() < 12 {
                        expected_resp.insert_balance_for_year(year, get_all_months_empty());
                        for (m, b) in current_year_balances {
                            expected_resp.insert_balance_opt(year, m, b);
                        }
                    }
                }
                None => {
                    expected_resp.insert_balance_for_year(year, get_all_months_empty());
                }
            }
        }
        let res_body = response.unwrap();
        assert_eq!(res_body, expected_resp);

        // Make sure the deletion removed it from db
        let saved = context.get_res(expected_resp.base.id).await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));

        if !expected_resp.is_empty() {
            // Updates all months that had balance
            for (year, month) in expected_resp.iter_months() {
                let saved_month = context.get_month(month, year).await;
                assert!(saved_month.is_ok());

                let saved_month = saved_month.unwrap();
                if !saved_month.resources.is_empty() {
                    // Since net_assets are computed from all resources' type
                    assert_ne!(saved_month.net_assets().total, 0);
                }
            }
        }

        // Delete the resource also computed net assets of the year
        let saved_years = context.get_years().await;
        assert!(saved_years.is_ok());
        let saved_years = saved_years.unwrap();
        for saved_year in saved_years {
            if let Some(last_month) = saved_year.get_last_month() {
                assert_eq!(saved_year.net_assets().total, last_month.net_assets().total);
            }
        }
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

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn updates_all_months_and_all_years(pool: SqlitePool) {
    let mut res = FinancialResourceYearly::new(
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
    );
    let years: [i32; 2] = [(1000..3000).fake(), (1000..3000).fake()];
    let months: [i16; 5] = [1, 2, 3, 8, 11];
    for month in months {
        let idx = (month % 2) as usize;
        let year = years[idx];
        res.insert_balance(year, month.try_into().unwrap(), (-1000000..1000000).fake());
    }

    check_delete(pool, Some(res), None).await;
}
