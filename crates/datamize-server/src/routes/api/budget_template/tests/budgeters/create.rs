use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{BudgeterConfig, Uuid};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::budget_template::tests::budgeters::testutils::TestContext;

#[derive(Debug, Deserialize, Serialize, Clone, fake::Dummy)]
struct CreateBody {
    pub name: String,
    pub payee_ids: Vec<Uuid>,
}

impl From<CreateBody> for BudgeterConfig {
    fn from(value: CreateBody) -> Self {
        Self::new(value.name, value.payee_ids)
    }
}

fn are_equal(a: &BudgeterConfig, b: &BudgeterConfig) {
    assert_eq!(a.name, b.name);
    assert_eq!(a.payee_ids, b.payee_ids);
}

async fn check_create(
    pool: SqlitePool,
    body: Option<CreateBody>,
    expected_status: StatusCode,
    expected_resp: Option<BudgeterConfig>,
) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/budgeter")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = expected_resp {
        let body: BudgeterConfig = serde_json::from_slice(&body).unwrap();
        are_equal(&body, &expected);

        let saved = context.get_budgeter_by_name(&expected.name).await.unwrap();
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
        context.set_budgeters(&[body.clone().into()]).await;
    }
    check_create(pool, Some(body), StatusCode::CONFLICT, None).await;
}
