use chrono::{Datelike, NaiveDate};
use datamize_domain::{FinancialResourceYearly, MonthNum};
use db_sqlite::balance_sheet::sabotage_resources_table;
use fake::{faker::chrono::en::Date, Fake};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::financial_resource::testutils::{
        correctly_stub_resources, transform_expected_resources, TestContext,
    },
    testutils::{assert_err, ErrorType},
};

async fn check_get_all_from_year(
    pool: SqlitePool,
    year: Option<i32>,
    expected_resp: Option<Vec<FinancialResourceYearly>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    if let Some(year) = year {
        context.insert_year(year).await;
    }
    let year = year.unwrap_or(Date().fake::<NaiveDate>().year());

    let expected_resp: Option<Vec<FinancialResourceYearly>> =
        correctly_stub_resources(expected_resp, [year, year]);

    if let Some(expected_resp) = &expected_resp {
        for r in expected_resp {
            for m in r
                .balance_per_month
                .keys()
                .cloned()
                .collect::<Vec<MonthNum>>()
            {
                context.insert_month(m, r.year).await;
            }
        }
        context.set_resources(expected_resp).await;
    }

    let response = context.service().get_all_fin_res_from_year(year).await;
    let expected_resp = transform_expected_resources(expected_resp);

    if let Some(expected_resp) = expected_resp {
        assert_eq!(response.unwrap(), expected_resp);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_list_when_no_year(pool: SqlitePool) {
    check_get_all_from_year(pool, None, Some(vec![]), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_that_is_in_db(pool: SqlitePool) {
    check_get_all_from_year(
        pool,
        Some(Date().fake::<NaiveDate>().year()),
        Some(fake::vec![FinancialResourceYearly; 3..6]),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_internal_when_db_corrupted(pool: SqlitePool) {
    sabotage_resources_table(&pool).await.unwrap();

    check_get_all_from_year(
        pool,
        Some(Date().fake::<NaiveDate>().year()),
        None,
        Some(ErrorType::Internal),
    )
    .await;
}
