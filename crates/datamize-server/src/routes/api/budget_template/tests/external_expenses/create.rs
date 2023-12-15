use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{ExpenseType, ExternalExpense, SubExpenseType};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::budget_template::tests::external_expenses::testutils::TestContext;

#[derive(Debug, Deserialize, Serialize, Clone, fake::Dummy)]
struct CreateBody {
    pub name: String,
    #[serde(rename = "type")]
    pub expense_type: ExpenseType,
    #[serde(rename = "sub_type")]
    pub sub_expense_type: SubExpenseType,
    pub projected_amount: i64,
}

impl From<CreateBody> for ExternalExpense {
    fn from(value: CreateBody) -> Self {
        Self::new(
            value.name,
            value.expense_type,
            value.sub_expense_type,
            value.projected_amount,
        )
    }
}

fn are_equal(a: &ExternalExpense, b: &ExternalExpense) {
    assert_eq!(a.name, b.name);
    assert_eq!(a.expense_type, b.expense_type);
    assert_eq!(a.sub_expense_type, b.sub_expense_type);
    assert_eq!(a.projected_amount, b.projected_amount);
}

async fn check_create(
    pool: SqlitePool,
    body: Option<CreateBody>,
    expected_status: StatusCode,
    expected_resp: Option<ExternalExpense>,
) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/external_expense")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = expected_resp {
        let body: ExternalExpense = serde_json::from_slice(&body).unwrap();
        are_equal(&body, &expected);

        let saved = context
            .get_external_expense_by_name(&expected.name)
            .await
            .unwrap();
        are_equal(&body, &saved);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_budgeter_config(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    check_create(
        pool,
        Some(body.clone()),
        StatusCode::CREATED,
        Some(body.into()),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_409_when_saving_rate_already_exists(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    {
        let context = TestContext::setup(pool.clone());
        context.set_external_expenses(&[body.clone().into()]).await;
    }
    check_create(pool, Some(body), StatusCode::CONFLICT, None).await;
}
