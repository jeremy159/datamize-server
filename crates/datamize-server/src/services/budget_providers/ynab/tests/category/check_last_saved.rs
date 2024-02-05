use chrono::{Local, Months, NaiveDate};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    budget_providers::ynab::tests::category::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

async fn check_check_last_saved(
    pool: SqlitePool,
    last_saved: Option<String>,
    expected_date: Option<NaiveDate>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool, Faker.fake(), Faker.fake()).await;

    if let Some(date) = last_saved.clone() {
        context.set_last_saved(date).await;
    }
    let delta_before = context.get_delta().await.unwrap();

    let response = context.service().check_last_saved().await;

    if let Some(date) = expected_date {
        let saved: NaiveDate = context.get_last_saved().await.parse().unwrap();

        assert_eq!(saved, date);

        if let Some(last_saved) = last_saved {
            if let Ok(last_saved) = last_saved.parse::<NaiveDate>() {
                let delta_after = context.get_delta().await;

                if last_saved != date {
                    assert_err(delta_after.unwrap_err().into(), Some(ErrorType::NotFound));
                } else {
                    assert_eq!(delta_before, delta_after.unwrap());
                }
            }
        }
    } else {
        assert_err(response.unwrap_err(), expected_err);
        let delta_after = context.get_delta().await.unwrap();

        assert_eq!(delta_before, delta_after);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn when_nothing_currently_saved_should_update_last_saved(pool: SqlitePool) {
    check_check_last_saved(pool, None, Some(Local::now().date_naive()), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn when_saved_date_is_the_same_month_as_current_should_not_update_last_saved(
    pool: SqlitePool,
) {
    check_check_last_saved(
        pool,
        Some(Local::now().date_naive().to_string()),
        Some(Local::now().date_naive()),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn when_saved_date_is_not_the_same_month_as_current_should_update_last_saved_and_delete_delta(
    pool: SqlitePool,
) {
    check_check_last_saved(
        pool,
        Some(
            Local::now()
                .date_naive()
                .checked_sub_months(Months::new(1))
                .unwrap()
                .to_string(),
        ),
        Some(Local::now().date_naive()),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn should_return_error_when_issue_occured_while_parsing_date(pool: SqlitePool) {
    check_check_last_saved(
        pool,
        Some(String::from("wrong date format")),
        None,
        Some(ErrorType::Internal),
    )
    .await;
}
