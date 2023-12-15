use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;
use ynab::{Account, AccountsDelta};

use crate::routes::budget_providers::ynab::tests::accounts::testutils::TestContext;

struct YnabData(AccountsDelta);

#[derive(Clone)]
struct DbData(Vec<Account>);

async fn check_get_all(
    pool: SqlitePool,
    ynab_data: YnabData,
    mut db_data: Option<DbData>,
    expected_status: StatusCode,
) {
    let context = TestContext::setup(pool, ynab_data.0.clone());

    if let Some(DbData(mut accounts)) = db_data.clone() {
        accounts.retain(|a| !a.deleted);
        context.set_accounts(&accounts).await;
    }

    let response = context
        .into_app()
        .oneshot(
            Request::builder()
                .uri("/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let mut body: Vec<Account> = serde_json::from_slice(&body).unwrap();
    let mut expected = ynab_data.0.accounts;
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
