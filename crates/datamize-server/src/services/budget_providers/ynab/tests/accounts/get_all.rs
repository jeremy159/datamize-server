use db_sqlite::budget_providers::ynab::sabotage_accounts_table;
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;
use ynab::{Account, AccountsDelta};

use crate::services::budget_providers::ynab::tests::accounts::testutils::{
    assert_err, ErrorType, TestContext,
};

struct YnabData(AccountsDelta);

#[derive(Clone)]
struct DbData(Vec<Account>);

async fn check_get_all(
    pool: SqlitePool,
    ynab_data: YnabData,
    db_data: Option<DbData>,
    expected_resp: Option<Vec<Account>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool, ynab_data.0.clone()).await;

    if let Some(DbData(mut accounts)) = db_data {
        accounts.retain(|a| !a.deleted);
        context.set_accounts(&accounts).await;
    }
    let delta_before = context.get_delta().await;

    let response = context.service().get_all_ynab_accounts().await;

    if let Some(mut expected_resp) = expected_resp {
        let mut res_body = response.unwrap();
        res_body.sort_by(|a, b| a.name.cmp(&b.name));
        expected_resp.retain(|a| !a.deleted);
        expected_resp.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(res_body, expected_resp);
        let delta_after = context.get_delta().await;

        assert_ne!(delta_before, delta_after);
    } else {
        assert_err(response.unwrap_err(), expected_err);
        let delta_after = context.get_delta().await;

        assert_eq!(delta_before, delta_after);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_empty_list_when_nothing_in_db(pool: SqlitePool) {
    let accounts_delta = AccountsDelta {
        accounts: vec![],
        ..Faker.fake()
    };
    check_get_all(pool, YnabData(accounts_delta), None, Some(vec![]), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    let accounts_delta = Faker.fake::<AccountsDelta>();
    let accounts = Faker.fake::<Vec<Account>>();
    let mut expected = accounts_delta.accounts.clone();
    expected.extend(accounts.clone());

    check_get_all(
        pool,
        YnabData(accounts_delta),
        Some(DbData(accounts.clone())),
        Some(expected),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn issue_with_db_should_not_update_saved_delta(pool: SqlitePool) {
    sabotage_accounts_table(&pool).await.unwrap();

    check_get_all(
        pool,
        YnabData(Faker.fake()),
        None,
        None,
        Some(ErrorType::Internal),
    )
    .await;
}
