use chrono::{Datelike, NaiveDate};
use datamize_domain::Month;
use db_sqlite::balance_sheet::sabotage_months_table;
use fake::{faker::chrono::en::Date, Fake};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::balance_sheet::tests::month::testutils::{
    assert_err, correctly_stub_months, transform_expected_months, ErrorType, TestContext,
};

async fn check_get_all(
    pool: SqlitePool,
    years: Option<(i32, i32)>,
    expected_resp: Option<Vec<Month>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    if let Some(years) = years {
        context.insert_year(years.0).await;
        context.insert_year(years.1).await;
    }
    let years = years.unwrap_or((
        Date().fake::<NaiveDate>().year(),
        Date().fake::<NaiveDate>().year(),
    ));
    let years: [i32; 2] = [years.0, years.1];

    let expected_resp: Option<Vec<Month>> = correctly_stub_months(expected_resp, years);

    if let Some(expected_resp) = &expected_resp {
        for m in expected_resp {
            context.set_month(m, m.year).await;
        }
    }

    let response = context.service().get_all_months().await;
    let expected_resp = transform_expected_months(expected_resp);

    if let Some(expected_resp) = expected_resp {
        assert_eq!(response.unwrap(), expected_resp);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_list_when_no_year(pool: SqlitePool) {
    check_get_all(pool, None, Some(vec![]), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_that_is_in_db(pool: SqlitePool) {
    check_get_all(
        pool,
        Some((
            Date().fake::<NaiveDate>().year(),
            Date().fake::<NaiveDate>().year(),
        )),
        Some(fake::vec![Month; 3..6]),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_internal_when_db_corrupted(pool: SqlitePool) {
    sabotage_months_table(&pool).await.unwrap();

    check_get_all(
        pool,
        Some((
            Date().fake::<NaiveDate>().year(),
            Date().fake::<NaiveDate>().year(),
        )),
        None,
        Some(ErrorType::Internal),
    )
    .await;
}
