use chrono::{Datelike, Local};
use datamize_domain::{
    BaseFinancialResource, FinancialResourceYearly, ResourcesToRefresh, Uuid, YearlyBalances,
};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;
use ynab::Account;

use crate::services::{
    balance_sheet::tests::refresh_resource::testutils::{correctly_stub_resources, TestContext},
    testutils::{assert_err, ErrorType},
};

async fn check_refresh(
    pool: SqlitePool,
    create_year: bool,
    resources_to_refresh: Option<ResourcesToRefresh>,
    ynab_accounts: Vec<Account>,
    resources: Vec<FinancialResourceYearly>,
    expected_resp: Option<Vec<Uuid>>,
    expected_err: Option<ErrorType>,
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
            for (_, month, _) in r.iter_balances() {
                if context.get_month_data(month, year).await.is_err() {
                    context.insert_month(month, year).await;
                }
            }
        }

        context.set_resources(&resources).await;
    }

    let response = context
        .service()
        .refresh_fin_res(resources_to_refresh)
        .await;

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        assert_eq!(res_body, expected_resp);

        if !res_body.is_empty() {
            // Creates current month that was not in db
            let saved_month = context
                .get_month(date.month().try_into().unwrap(), year)
                .await;
            assert!(saved_month.is_ok());

            let saved_month = saved_month.unwrap();
            // Since net_assets are computed from all resources' type
            assert_ne!(saved_month.net_assets().total, 0);
        }
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_no_year(pool: SqlitePool) {
    check_refresh(
        pool,
        false,
        None,
        fake::vec![Account; 3..6],
        fake::vec![FinancialResourceYearly; 3..6],
        None,
        Some(ErrorType::NotFound),
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
        Some(vec![res_id]),
        None,
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
        Some(vec![]),
        None,
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
        Some(vec![]),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn only_update_when_different_balance(pool: SqlitePool) {
    let ynab_account: Account = Faker.fake();
    let mut resource = FinancialResourceYearly::new(
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Some(vec![ynab_account.id]),
        Faker.fake(),
    );
    let current_date = Local::now().date_naive();
    let month = current_date.month().try_into().unwrap();
    let year = current_date.year();
    resource.insert_balance(year, month, ynab_account.balance.abs());

    check_refresh(
        pool,
        true,
        None,
        vec![ynab_account],
        vec![resource],
        Some(vec![]),
        None,
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
        Some(ResourcesToRefresh { ids: vec![res_id] }),
        ynab_accounts,
        vec![res, res2],
        Some(vec![res_id]),
        None,
    )
    .await;
}

//TODO: Add test for external accounts
