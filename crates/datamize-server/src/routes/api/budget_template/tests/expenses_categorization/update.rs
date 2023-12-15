use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::ExpenseCategorization;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::budget_template::tests::expenses_categorization::testutils::TestContext;

fn are_equal(a: &ExpenseCategorization, b: &ExpenseCategorization) {
    assert_eq!(a.name, b.name);
    assert_eq!(a.expense_type, b.expense_type);
    assert_eq!(a.sub_expense_type, b.sub_expense_type);
}

async fn check_update(
    pool: SqlitePool,
    req_body: Option<ExpenseCategorization>,
    expected_status: StatusCode,
    expected_resp: Option<ExpenseCategorization>,
) {
    let context = TestContext::setup(pool);

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_expenses_categorization(&[expected_resp]).await;
    }

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/expense_categorization/{:?}",
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

    if let Some(expected) = expected_resp {
        let body: ExpenseCategorization = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);
        if let Some(req_body) = req_body {
            // Make sure the update is persisted in db
            let saved = context
                .get_expense_categorization(req_body.id)
                .await
                .unwrap();
            are_equal(&req_body, &saved);
        }
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_update(pool, Some(Faker.fake()), StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let body: ExpenseCategorization = Faker.fake();
    let expected_resp = ExpenseCategorization { ..body.clone() };

    check_update(pool, Some(body), StatusCode::OK, Some(expected_resp)).await;
}
