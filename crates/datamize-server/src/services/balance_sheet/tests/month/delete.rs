use datamize_domain::{FinancialResourceMonthly, Month};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::month::testutils::{
        correctly_stub_month, transform_expected_month, TestContext,
    },
    testutils::{assert_err, ErrorType},
};

async fn check_delete(
    pool: SqlitePool,
    create_year: bool,
    expected_resp: Option<Month>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    let year = expected_resp.clone().unwrap_or_else(|| Faker.fake()).year;
    if create_year {
        context.insert_year(year).await;
    }

    let expected_resp = correctly_stub_month(expected_resp);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_month(&expected_resp, year).await;
    }
    let month = expected_resp.clone().unwrap_or_else(|| Faker.fake()).month;

    let response = context.service().delete_month(month, year).await;

    let expected_resp = transform_expected_month(expected_resp);

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        assert_eq!(res_body, expected_resp);

        // Make sure the deletion removed it from db
        let saved = context
            .get_month(expected_resp.month, expected_resp.year)
            .await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));

        // Make sure the deletion removed net totals of the month from db
        let saved_net_totals = context.get_net_totals(expected_resp.id).await;
        assert_eq!(saved_net_totals, Ok(vec![]));
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
async fn does_not_delete_same_month_of_different_year(pool: SqlitePool) {
    let month: Month = Faker.fake();
    let same_month_other_year = Month {
        year: month.year + 1,
        month: month.month,
        ..Faker.fake()
    };
    let context = TestContext::setup(pool.clone());
    context.insert_year(same_month_other_year.year).await;
    let mut same_month_other_year = Month {
        resources: same_month_other_year
            .resources
            .into_iter()
            .map(|r| FinancialResourceMonthly {
                year: same_month_other_year.year,
                month: same_month_other_year.month,
                ..r
            })
            .collect(),
        ..same_month_other_year
    };
    same_month_other_year
        .resources
        .sort_by(|a, b| a.base.name.cmp(&b.base.name));

    context
        .set_month(&same_month_other_year, same_month_other_year.year)
        .await;

    check_delete(pool, true, Some(month), None).await;
    // Make sure the deletion did not remove the other month
    let mut saved = context
        .get_month(same_month_other_year.month, same_month_other_year.year)
        .await
        .unwrap();
    saved
        .resources
        .sort_by(|a, b| a.base.name.cmp(&b.base.name));
    assert_eq!(saved, same_month_other_year);
}
