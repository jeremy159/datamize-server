use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{Incomes, SavingRate, Savings};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;
use tower::ServiceExt;
use ynab::TransactionDetail;

use crate::routes::api::balance_sheet::testutils::TestContext;

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
    req_body: Option<SavingRate>,
    expected_status: StatusCode,
    expected_resp: Option<SavingRate>,
) {
    let context = TestContext::setup(pool);

    if create_year {
        let year = req_body.clone().expect("missing body to create year").year;
        context.insert_year(year).await;
    }

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_saving_rates(&[expected_resp]).await;
    }

    let transactions = fake::vec![TransactionDetail; 1..5];
    context.set_transactions(&transactions).await;

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/saving_rates/{:?}",
                    req_body
                        .clone()
                        .expect("missing body to fetch saving rate")
                        .id
                ))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(mut expected_resp) = expected_resp {
        expected_resp.compute_totals(&transactions);
        let body: SavingRate = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected_resp);
        if let Some(req_body) = req_body {
            // Make sure the requested body is not equal to the saving rate that was in the db. I.e. compute totals should have updated something
            assert_ne!(req_body, expected_resp);

            // Make sure the update is persisted in db
            let saved = context
                .get_saving_rate_by_name(&expected_resp.name)
                .await
                .unwrap();
            are_equal(&req_body, &saved);
        }
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_no_year(pool: SqlitePool) {
    check_update(pool, false, Some(Faker.fake()), StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_update(pool, true, Some(Faker.fake()), StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let body: SavingRate = Faker.fake();
    let expected_resp = SavingRate {
        savings: Savings {
            total: Faker.fake(),
            ..body.savings.clone()
        },
        incomes: Incomes {
            total: Faker.fake(),
            ..body.incomes.clone()
        },
        ..body.clone()
    };

    check_update(pool, true, Some(body), StatusCode::OK, Some(expected_resp)).await;
}
