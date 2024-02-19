use datamize_domain::{Incomes, SaveSavingRate, SavingRate, Savings};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;
use ynab::TransactionDetail;

use crate::services::{
    balance_sheet::{tests::saving_rate::testutils::TestContext, SavingRateServiceExt},
    testutils::{assert_err, ErrorType},
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

async fn check_update(
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

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_saving_rates(&[expected_resp]).await;
    }

    let transactions = fake::vec![TransactionDetail; 1..5];
    context.set_transactions(&transactions).await;

    let response = context
        .service()
        .update_saving_rate(new_saving_rate.clone())
        .await;

    if let Some(mut expected_resp) = expected_resp {
        expected_resp.compute_totals(&transactions);
        let res_body = response.unwrap();
        assert_eq!(res_body, expected_resp);

        // Make sure the requested body is not equal to the saving rate that was in the db. I.e. compute totals should have updated something
        assert_ne!(
            Into::<SavingRate>::into(new_saving_rate.clone()),
            expected_resp
        );

        // Make sure the update is persisted in db
        let saved = context
            .get_saving_rate_by_name(&expected_resp.name)
            .await
            .unwrap();
        are_equal(&new_saving_rate.into(), &saved);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_no_year(pool: SqlitePool) {
    check_update(pool, false, Faker.fake(), None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_nothing_in_db(pool: SqlitePool) {
    check_update(pool, true, Faker.fake(), None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let body: SaveSavingRate = Faker.fake();
    let expected_resp = SavingRate {
        savings: Savings {
            total: Faker.fake(),
            category_ids: body.savings.category_ids.clone(),
            extra_balance: body.savings.extra_balance,
        },
        incomes: Incomes {
            total: Faker.fake(),
            payee_ids: body.incomes.payee_ids.clone(),
            extra_balance: body.incomes.extra_balance,
        },
        id: body.id,
        name: body.name.clone(),
        year: body.year,
        employee_contribution: body.employee_contribution,
        employer_contribution: body.employer_contribution,
        mortgage_capital: body.mortgage_capital,
    };

    check_update(pool, true, body, Some(expected_resp), None).await;
}
