use std::collections::BTreeMap;

use datamize_domain::FinancialResourceYearly;
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::financial_resource::testutils::{correctly_stub_resource, TestContext},
    testutils::{assert_err, ErrorType},
};

async fn check_delete(
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
        .delete_fin_res(
            expected_resp
                .clone()
                .unwrap_or_else(|| Faker.fake())
                .base
                .id,
        )
        .await;

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        assert_eq!(res_body, expected_resp);

        // Make sure the deletion removed it from db
        let saved = context.get_res(expected_resp.base.id).await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));

        if !expected_resp.balance_per_month.is_empty() {
            // Updates all months that had balance
            for m in expected_resp.balance_per_month.keys() {
                let saved_month = context.get_month(*m, expected_resp.year).await;
                assert!(saved_month.is_ok());

                let saved_month = saved_month.unwrap();
                if !saved_month.resources.is_empty() {
                    // Since net_assets are computed from all resources' type
                    assert_ne!(saved_month.net_assets.total, 0);
                }
            }
        }

        // Delete the resource also computed net assets of the year
        let saved_year = context.get_year(expected_resp.year).await;
        assert!(saved_year.is_ok());
        let saved_year = saved_year.unwrap();
        if let Some(last_month) = saved_year.get_last_month() {
            assert_eq!(saved_year.net_assets.total, last_month.net_assets.total);
        }
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_no_year(pool: SqlitePool) {
    check_delete(pool, false, None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_nothing_in_db(pool: SqlitePool) {
    check_delete(pool, true, None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_deletion(pool: SqlitePool) {
    check_delete(pool, true, Some(Faker.fake()), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn updates_all_months(pool: SqlitePool) {
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(
        TryFrom::<i16>::try_from(1).unwrap(),
        (-1000000..1000000).fake(),
    );
    balance_per_month.insert(
        TryFrom::<i16>::try_from(2).unwrap(),
        (-1000000..1000000).fake(),
    );
    balance_per_month.insert(
        TryFrom::<i16>::try_from(3).unwrap(),
        (-1000000..1000000).fake(),
    );
    balance_per_month.insert(
        TryFrom::<i16>::try_from(7).unwrap(),
        (-1000000..1000000).fake(),
    );
    balance_per_month.insert(
        TryFrom::<i16>::try_from(11).unwrap(),
        (-1000000..1000000).fake(),
    );
    let res = FinancialResourceYearly {
        balance_per_month,
        ..Faker.fake()
    };

    check_delete(pool, true, Some(res), None).await;
}
