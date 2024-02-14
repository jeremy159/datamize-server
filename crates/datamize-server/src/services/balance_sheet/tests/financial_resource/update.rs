use std::collections::{BTreeMap, HashSet};

use chrono::{Datelike, NaiveDate};
use datamize_domain::{FinancialResourceYearly, UpdateResource, YearlyBalances};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::financial_resource::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

async fn check_update(
    pool: SqlitePool,
    updated_res: UpdateResource,
    db_data: Option<FinancialResourceYearly>,
    expected_resp: Option<FinancialResourceYearly>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);
    let mut checked_years = HashSet::<i32>::new();

    if let Some(ref db_data) = db_data {
        // Create all months and years
        for (year, month) in db_data.iter_months() {
            if !checked_years.contains(&year) {
                checked_years.insert(year);
                context.insert_year(year).await;
            }
            context.insert_month(month, year).await;
        }
        context.set_resource(db_data).await;
    }

    let response = context.service().update_fin_res(updated_res.clone()).await;

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        assert_eq!(res_body, expected_resp);

        if let Some(db_data) = db_data {
            // Make sure the requested body is not equal to the resource that was in the db. I.e. new balance per month should have updated something
            assert_ne!(updated_res.base.name, db_data.base.name,);
        }

        // Make sure the update is persisted in db
        let saved = context.get_res(expected_resp.base.id).await.unwrap();
        assert_eq!(updated_res.base.name, saved.base.name);
        assert_eq!(expected_resp.balances, saved.balances);

        if !expected_resp.is_empty() {
            // Creates all months that were not created
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

        // Updating the resource also computed net assets of the year
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
    check_update(pool, Faker.fake(), None, None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let mut db_data: FinancialResourceYearly = Faker.fake();
    db_data.clear_all_balances();
    let current_date = Faker.fake::<NaiveDate>();
    let month = current_date.month().try_into().unwrap();
    let year = current_date.year();
    db_data.insert_balance(year, month, (-1000000..1000000).fake());

    let mut body = UpdateResource {
        base: db_data.clone().base,
        balances: BTreeMap::new(),
    };
    body.base.name = Faker.fake();
    body.insert_balance_opt(year, month, Some((-1000000..1000000).fake()));

    let body_cloned = body.clone();
    let mut expected_resp = FinancialResourceYearly {
        base: body_cloned.base,
        balances: BTreeMap::new(),
    };

    for (year, month, balance) in body.iter_all_balances() {
        if let Some(balance) = balance {
            expected_resp.insert_balance(year, month, balance);
        }
    }

    check_update(pool, body, Some(db_data), Some(expected_resp), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn updated_resource_updates_all_months_and_all_years(pool: SqlitePool) {
    let mut db_data: FinancialResourceYearly = Faker.fake();
    db_data.clear_all_balances();
    let mut body = UpdateResource {
        base: db_data.clone().base,
        balances: BTreeMap::new(),
    };
    body.base.name = Faker.fake();
    let years: [i32; 2] = [(1000..3000).fake(), (1000..3000).fake()];
    let months: [i16; 5] = [1, 2, 3, 8, 11];
    for month in months {
        let idx = (month % 2) as usize;
        let year = years[idx];
        let month = month.try_into().unwrap();
        db_data.insert_balance(year, month, (-1000000..1000000).fake());
        body.insert_balance_opt(year, month, Some((-1000000..1000000).fake()));
    }

    let body_cloned = body.clone();
    let mut expected_resp = FinancialResourceYearly {
        base: body_cloned.base,
        balances: BTreeMap::new(),
    };

    for (year, month, balance) in body.iter_all_balances() {
        if let Some(balance) = balance {
            expected_resp.insert_balance(year, month, balance);
        }
    }

    check_update(pool, body, Some(db_data), Some(expected_resp), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn updated_resource_creates_or_deletes_balance_of_provided_month(pool: SqlitePool) {
    let mut db_data: FinancialResourceYearly = Faker.fake();
    db_data.clear_all_balances();
    let mut body = UpdateResource {
        base: db_data.clone().base,
        balances: BTreeMap::new(),
    };
    body.base.name = Faker.fake();
    let years: [i32; 2] = [(1000..3000).fake(), (1000..3000).fake()];
    let months: [i16; 5] = [1, 2, 3, 8, 11];
    for month in months {
        let idx = (month % 2) as usize;
        let year = years[idx];
        let month = month.try_into().unwrap();
        db_data.insert_balance(year, month, (-1000000..1000000).fake());
        body.insert_balance(year, month, (-1000000..1000000).fake());
    }
    // Attempt on a non-existing month
    body.insert_balance_opt(years[1], 4_i16.try_into().unwrap(), None);
    // Delete balance of march
    body.insert_balance_opt(years[1], 3_i16.try_into().unwrap(), None);
    // Create balance for may (new month)
    body.insert_balance(
        years[1],
        5_i16.try_into().unwrap(),
        (-1000000..1000000).fake(),
    );

    let body_cloned = body.clone();
    let mut expected_resp = FinancialResourceYearly {
        base: body_cloned.base,
        balances: BTreeMap::new(),
    };

    for (year, month, balance) in body.iter_all_balances() {
        if let Some(balance) = balance {
            expected_resp.insert_balance(year, month, balance);
        }
    }

    check_update(pool, body, Some(db_data), Some(expected_resp), None).await;
}
