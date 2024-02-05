use db_sqlite::budget_providers::ynab::sabotage_transactions_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use ynab::{TransactionDetail, TransactionsDetailDelta};

use crate::services::{
    budget_providers::ynab::tests::transaction::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

struct YnabData(TransactionsDetailDelta);

#[derive(Clone)]
struct DbData(Vec<TransactionDetail>);

async fn check_refresh(
    pool: SqlitePool,
    ynab_data: YnabData,
    mut db_data: Option<DbData>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool, ynab_data.0.clone()).await;

    if let Some(DbData(ref transactions)) = db_data {
        context.set_transactions(transactions).await;
    }
    let delta_before = context.get_delta().await;

    let response = context.service().refresh_saved_transactions().await;

    if expected_err.is_some() {
        assert_err(response.unwrap_err(), expected_err);
        let delta_after = context.get_delta().await;

        assert_eq!(delta_before, delta_after);
    } else {
        let delta_after = context.get_delta().await;

        assert_ne!(delta_before, delta_after);

        let mut expected = ynab_data.0.transactions;
        if let Some(DbData(ref mut saved)) = db_data {
            expected.append(saved);
        }
        expected.sort_by_key(|t| t.base.amount);
        let mut saved = context.get_transactions().await;
        saved.sort_by_key(|t| t.base.amount);
        assert_eq!(saved, expected);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_empty_list_when_nothing_in_db(pool: SqlitePool) {
    let transactions_delta = TransactionsDetailDelta {
        transactions: vec![],
        ..Faker.fake()
    };
    check_refresh(pool, YnabData(transactions_delta), None, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    let transactions_delta = Faker.fake::<TransactionsDetailDelta>();
    let transactions = Faker.fake::<Vec<TransactionDetail>>();

    check_refresh(
        pool,
        YnabData(transactions_delta),
        Some(DbData(transactions)),
        None,
    )
    .await;
}

// FIXME: For some reasons sometimes the test fails... Might be related to the redis test mock
// #[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn issue_with_db_should_not_update_saved_delta(pool: SqlitePool) {
    sabotage_transactions_table(&pool).await.unwrap();

    check_refresh(
        pool,
        YnabData(Faker.fake()),
        None,
        Some(ErrorType::Internal),
    )
    .await;
}
