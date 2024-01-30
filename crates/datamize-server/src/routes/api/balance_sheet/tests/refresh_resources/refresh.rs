use std::collections::BTreeMap;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Datelike, Local};
use datamize_domain::{BaseFinancialResource, FinancialResourceYearly, Uuid};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;
use ynab::Account;

use crate::routes::api::balance_sheet::tests::refresh_resources::testutils::{
    correctly_stub_resources, TestContext,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CreateBody {
    pub ids: Vec<Uuid>,
}

async fn check_refresh(
    pool: SqlitePool,
    create_year: bool,
    body: Option<CreateBody>,
    ynab_accounts: Vec<Account>,
    resources: Vec<FinancialResourceYearly>,
    expected_status: StatusCode,
    expected_resp: Option<Vec<Uuid>>,
) {
    let context = match create_year {
        true => TestContext::setup(pool, 1, ynab_accounts),
        false => TestContext::setup(pool, 0, ynab_accounts),
    }
    .await;

    let date = Local::now().date_naive();
    let year = date.year();

    if create_year {
        context.insert_year(year).await;

        let resources = correctly_stub_resources(resources, year);

        // Create all months
        for r in &resources {
            for m in r.balance_per_month.keys() {
                if context.get_month_data(*m, year).await.is_err() {
                    context.insert_month(*m, year).await;
                }
            }
        }

        context.set_resources(&resources).await;
    }

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .header("Content-Type", "application/json")
                .uri("/resources/refresh")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = expected_resp {
        let body: Vec<Uuid> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);

        if !body.is_empty() {
            // Creates current month that was not in db
            let saved_month = context
                .get_month(date.month().try_into().unwrap(), year)
                .await;
            assert!(saved_month.is_ok());

            let saved_month = saved_month.unwrap();
            // Since net_assets are computed from all resources' type
            assert_ne!(saved_month.net_assets.total, 0);
        }
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_no_year(pool: SqlitePool) {
    check_refresh(
        pool,
        false,
        None,
        fake::vec![Account; 3..6],
        fake::vec![FinancialResourceYearly; 3..6],
        StatusCode::NOT_FOUND,
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_refreshed_ids(pool: SqlitePool) {
    let ynab_accounts = fake::vec![Account; 3..6];
    let res = FinancialResourceYearly {
        base: BaseFinancialResource {
            ynab_account_ids: Some(ynab_accounts.clone().into_iter().map(|ya| ya.id).collect()),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let res_id = res.base.id;

    check_refresh(
        pool,
        true,
        None,
        ynab_accounts,
        vec![res],
        StatusCode::OK,
        Some(vec![res_id]),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_ids_when_no_ynab_accounts(pool: SqlitePool) {
    check_refresh(
        pool,
        true,
        None,
        vec![],
        fake::vec![FinancialResourceYearly; 3..6],
        StatusCode::OK,
        Some(vec![]),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_ids_when_no_ynab_accounts_linked_to_returned_resources(pool: SqlitePool) {
    let ynab_accounts = fake::vec![Account; 1..3];
    let mut resources = fake::vec![FinancialResourceYearly; 3..6];
    for r in &mut resources {
        let filtered_ids = r.base.ynab_account_ids.clone().map(|yai| {
            yai.into_iter()
                .filter(|&yai| !ynab_accounts.iter().any(|ya| ya.id == yai))
                .collect()
        });
        r.base.ynab_account_ids = filtered_ids;
    }

    check_refresh(
        pool,
        true,
        None,
        ynab_accounts,
        resources,
        StatusCode::OK,
        Some(vec![]),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn only_update_when_different_balance(pool: SqlitePool) {
    let ynab_account: Account = Faker.fake();
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(
        Local::now().date_naive().month().try_into().unwrap(),
        ynab_account.balance.abs(),
    );
    let resource = FinancialResourceYearly {
        balance_per_month,
        base: BaseFinancialResource {
            ynab_account_ids: Some(vec![ynab_account.id]),
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    check_refresh(
        pool,
        true,
        None,
        vec![ynab_account],
        vec![resource],
        StatusCode::OK,
        Some(vec![]),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn only_update_requested_ids(pool: SqlitePool) {
    let ynab_accounts = fake::vec![Account; 3..6];
    let res = FinancialResourceYearly {
        base: BaseFinancialResource {
            ynab_account_ids: Some(
                ynab_accounts
                    .clone()
                    .into_iter()
                    .map(|ya| ya.id)
                    .take(2)
                    .collect(),
            ),
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let res2 = FinancialResourceYearly {
        base: BaseFinancialResource {
            ynab_account_ids: Some(
                ynab_accounts
                    .clone()
                    .into_iter()
                    .map(|ya| ya.id)
                    .skip(2)
                    .collect(),
            ),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let res_id = res.base.id;

    check_refresh(
        pool,
        true,
        Some(CreateBody { ids: vec![res_id] }),
        ynab_accounts,
        vec![res, res2],
        StatusCode::OK,
        Some(vec![res_id]),
    )
    .await;
}
