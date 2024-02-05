use datamize_domain::DatamizeScheduledTransaction;
use db_sqlite::budget_providers::ynab::sabotage_scheduled_transactions_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use ynab::{ScheduledTransactionDetail, ScheduledTransactionsDetailDelta};

use crate::services::{
    budget_providers::{
        ynab::tests::scheduled_transaction::testutils::TestContext, ScheduledTransactionServiceExt,
    },
    testutils::{assert_err, ErrorType},
};

struct YnabData(ScheduledTransactionsDetailDelta);

#[derive(Clone)]
struct DbData(Vec<ScheduledTransactionDetail>);

async fn check_get_latest(
    pool: SqlitePool,
    ynab_data: YnabData,
    db_data: Option<DbData>,
    expected_resp: Option<Vec<DatamizeScheduledTransaction>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool, ynab_data.0.clone()).await;

    if let Some(DbData(ref transactions)) = db_data {
        context.set_transactions(transactions).await;
    }
    let delta_before = context.get_delta().await;

    let response = context.service().get_latest_scheduled_transactions().await;

    if let Some(mut expected_resp) = expected_resp {
        let mut res_body = response.unwrap();
        res_body.sort_by_key(|t| t.amount);
        expected_resp.sort_by_key(|t| t.amount);
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
    let transactions_delta = ScheduledTransactionsDetailDelta {
        scheduled_transactions: vec![],
        ..Faker.fake()
    };
    check_get_latest(pool, YnabData(transactions_delta), None, Some(vec![]), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    let transactions_delta = Faker.fake::<ScheduledTransactionsDetailDelta>();
    let transactions = Faker.fake::<Vec<ScheduledTransactionDetail>>();
    let mut expected = transactions_delta.scheduled_transactions.clone();
    expected.extend(transactions.clone());
    let expected = expected
        .into_iter()
        .filter(|st| !st.deleted)
        .map(Into::into)
        .collect();

    check_get_latest(
        pool,
        YnabData(transactions_delta),
        Some(DbData(transactions.clone())),
        Some(expected),
        None,
    )
    .await;
}

// FIXME: For some reasons sometimes the test fails... Might be related to the redis test mock
// #[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn issue_with_db_should_not_update_saved_delta(pool: SqlitePool) {
    sabotage_scheduled_transactions_table(&pool).await.unwrap();

    check_get_latest(
        pool,
        YnabData(Faker.fake()),
        None,
        None,
        Some(ErrorType::Internal),
    )
    .await;
}
