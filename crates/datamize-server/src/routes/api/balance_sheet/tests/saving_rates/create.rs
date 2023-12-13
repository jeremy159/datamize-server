use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{Incomes, SavingRate, Savings, Uuid};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;
use ynab::TransactionDetail;

use crate::routes::api::balance_sheet::tests::saving_rates::testutils::TestContext;

#[derive(Debug, Deserialize, Serialize, Clone, fake::Dummy)]
struct CreateBody {
    pub id: Uuid,
    pub name: String,
    pub year: i32,
    pub savings: SaveSavings,
    pub employer_contribution: i64,
    pub employee_contribution: i64,
    pub mortgage_capital: i64,
    pub incomes: SaveIncomes,
}

#[derive(Debug, Deserialize, Serialize, Clone, fake::Dummy)]
struct SaveSavings {
    pub category_ids: Vec<Uuid>,
    pub extra_balance: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, fake::Dummy)]
struct SaveIncomes {
    pub payee_ids: Vec<Uuid>,
    pub extra_balance: i64,
}

impl From<CreateBody> for SavingRate {
    fn from(value: CreateBody) -> Self {
        Self {
            id: value.id,
            name: value.name,
            year: value.year,
            employee_contribution: value.employee_contribution,
            employer_contribution: value.employer_contribution,
            mortgage_capital: value.mortgage_capital,
            savings: Savings {
                category_ids: value.savings.category_ids,
                extra_balance: value.savings.extra_balance,
                total: 0,
            },
            incomes: Incomes {
                payee_ids: value.incomes.payee_ids,
                extra_balance: value.incomes.extra_balance,
                total: 0,
            },
        }
    }
}

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
    body: Option<CreateBody>,
    expected_status: StatusCode,
    expected_resp: Option<SavingRate>,
) {
    let context = TestContext::setup(pool);

    if create_year {
        let year = body.clone().expect("missing body to create year").year;
        context.insert_year(year).await;
    }

    let transactions = fake::vec![TransactionDetail; 1..5];
    context.set_transactions(&transactions).await;

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/saving_rates")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(mut expected_resp) = expected_resp {
        expected_resp.compute_totals(&transactions);
        let body: SavingRate = serde_json::from_slice(&body).unwrap();
        are_equal(&body, &expected_resp);

        let saved = context
            .get_saving_rate_by_name(&expected_resp.name)
            .await
            .unwrap();
        are_equal(&body, &saved);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_saving_rate(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    check_create(
        pool,
        true,
        Some(body.clone()),
        StatusCode::CREATED,
        Some(body.into()),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_year_does_not_exist(pool: SqlitePool) {
    check_create(pool, false, Some(Faker.fake()), StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_409_when_saving_rate_already_exists(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    {
        let context = TestContext::setup(pool.clone());
        let year = body.year;
        context.insert_year(year).await;
        context.set_saving_rates(&[body.clone().into()]).await;
    }
    check_create(pool, false, Some(body), StatusCode::CONFLICT, None).await;
}

// TODO: check for 415 and missing/wrong body params...
