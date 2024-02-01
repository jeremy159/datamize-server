use datamize_domain::{SaveSavingRate, SavingRate};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use ynab::TransactionDetail;

use crate::services::balance_sheet::{
    tests::saving_rate::testutils::{assert_err, ErrorType, TestContext},
    SavingRateServiceExt,
};

fn are_equal(a: &SavingRate, b: &SavingRate) {
    assert_eq!(a.name, b.name);
    assert_eq!(a.year, b.year);
    assert_eq!(a.employee_contribution, b.employee_contribution);
    assert_eq!(a.employer_contribution, b.employer_contribution);
    assert_eq!(a.mortgage_capital, b.mortgage_capital);
    assert_eq!(a.savings.category_ids, b.savings.category_ids);
    assert_eq!(a.savings.extra_balance, b.savings.extra_balance);
    assert_eq!(a.incomes.payee_ids, b.incomes.payee_ids);
    assert_eq!(a.incomes.extra_balance, b.incomes.extra_balance);
}

async fn check_create(
    pool: SqlitePool,
    create_year: bool,
    new_saving_rate: SaveSavingRate,
    expected_resp: Option<SavingRate>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool).await;

    if create_year {
        let year = new_saving_rate.year;
        context.insert_year(year).await;
    }

    let transactions = fake::vec![TransactionDetail; 1..5];
    context.set_transactions(&transactions).await;

    let response = context.service().create_saving_rate(new_saving_rate).await;

    if let Some(mut expected_resp) = expected_resp {
        expected_resp.compute_totals(&transactions);
        let res_body = response.unwrap();
        are_equal(&res_body, &expected_resp);

        let saved = context
            .get_saving_rate_by_name(&expected_resp.name)
            .await
            .unwrap();
        are_equal(&res_body, &saved);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_saving_rate(pool: SqlitePool) {
    let body: SaveSavingRate = Faker.fake();
    check_create(pool, true, body.clone(), Some(body.into()), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_year_does_not_exist(pool: SqlitePool) {
    check_create(pool, false, Faker.fake(), None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_already_exists_when_saving_rate_already_exists(pool: SqlitePool) {
    let body: SaveSavingRate = Faker.fake();
    {
        let context = TestContext::setup(pool.clone()).await;
        let year = body.year;
        context.insert_year(year).await;
        context.set_saving_rates(&[body.clone().into()]).await;
    }
    check_create(pool, false, body, None, Some(ErrorType::AlreadyExist)).await;
}
