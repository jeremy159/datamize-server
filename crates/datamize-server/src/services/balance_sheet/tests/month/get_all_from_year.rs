use chrono::{Datelike, NaiveDate};
use datamize_domain::Month;
use fake::{faker::chrono::en::Date, Fake};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::month::testutils::{
        correctly_stub_months, transform_expected_months, TestContext,
    },
    testutils::{assert_err, ErrorType},
};

async fn check_get_all_from_year(
    pool: SqlitePool,
    year: Option<i32>,
    expected_resp: Option<Vec<Month>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    if let Some(year) = year {
        context.insert_year(year).await;
    }
    let year = year.unwrap_or(Date().fake::<NaiveDate>().year());

    let expected_resp: Option<Vec<Month>> = correctly_stub_months(expected_resp, [year, year]);

    if let Some(expected_resp) = &expected_resp {
        for m in expected_resp {
            context.set_month(m, m.year).await;
        }
    }

    let response = context.service().get_all_months_from_year(year).await;
    let expected_resp = transform_expected_months(expected_resp);

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
        Some(fake::vec![Month; 3..6]),
        None,
    )
    .await;
}
