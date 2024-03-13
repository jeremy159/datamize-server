use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fake::{Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;
use ynab::{Payee, PayeesDelta};

use crate::routes::api::budget_providers::ynab::tests::payees::testutils::TestContext;

struct YnabData(PayeesDelta);

#[derive(Clone)]
struct DbData(Vec<Payee>);

async fn check_get_all(
    pool: SqlitePool,
    ynab_data: YnabData,
    mut db_data: Option<DbData>,
    expected_status: StatusCode,
) {
    let context = TestContext::setup(pool, ynab_data.0.clone()).await;

    if let Some(DbData(mut payees)) = db_data.clone() {
        payees.retain(|a| !a.deleted);
        context.set_payees(&payees).await;
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri("/payees")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let mut body: Vec<Payee> = serde_json::from_slice(&body).unwrap();
    let mut expected = ynab_data.0.payees;
    if let Some(DbData(saved)) = &mut db_data {
        expected.append(saved);
    }
    body.sort_by(|a, b| a.name.cmp(&b.name));
    expected.retain(|a| !a.deleted);
    expected.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(body, expected);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_200_when_nothing_in_db(pool: SqlitePool) {
    check_get_all(pool, YnabData(Faker.fake()), None, StatusCode::OK).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    check_get_all(
        pool,
        YnabData(Faker.fake()),
        Some(DbData(Faker.fake())),
        StatusCode::OK,
    )
    .await;
}
