use fake::{Fake, Faker};
use rand::seq::IteratorRandom;
use sqlx::SqlitePool;
use ynab::{
    Category, ScheduledSubTransaction, ScheduledTransactionDetail, ScheduledTransactionsDetailDelta,
};

use crate::services::budget_template::{
    tests::template_transaction::testutils::{count_sub_transaction_ids, TestContext},
    TemplateTransactionServiceExt,
};

struct YnabData(ScheduledTransactionsDetailDelta);

struct DbData(Vec<Category>);

async fn check_get(
    pool: SqlitePool,
    ynab_calls: usize,
    ynab_data: YnabData,
    db_data: Option<DbData>,
) {
    let context = TestContext::setup(pool, ynab_data.0, ynab_calls).await;

    if let Some(DbData(categories)) = db_data {
        context.set_categories(&categories).await;
    }

    let response = context.into_service().get_template_transactions().await;

    // We don't really care what's the answer, as long as it is able to parse it
    response.unwrap();
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_200_when_nothing_in_db(pool: SqlitePool) {
    let ynab_scheduled_transactions = ScheduledTransactionsDetailDelta {
        scheduled_transactions: vec![ScheduledTransactionDetail {
            subtransactions: fake::vec![ScheduledSubTransaction; 1],
            ..Faker.fake()
        }],
        ..Faker.fake()
    };
    let ynab_calls = count_sub_transaction_ids(&ynab_scheduled_transactions);

    check_get(
        pool,
        ynab_calls,
        YnabData(ynab_scheduled_transactions),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    let ynab_scheduled_transactions = ScheduledTransactionsDetailDelta {
        scheduled_transactions: fake::vec![ScheduledTransactionDetail; 1..10],
        ..Faker.fake()
    };
    let ynab_calls = count_sub_transaction_ids(&ynab_scheduled_transactions);

    check_get(
        pool,
        ynab_calls,
        YnabData(ynab_scheduled_transactions),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn does_not_call_ynab_api_when_category_is_in_db(pool: SqlitePool) {
    let mut ynab_scheduled_transactions = ScheduledTransactionsDetailDelta {
        scheduled_transactions: fake::vec![ScheduledTransactionDetail; 1..10],
        ..Faker.fake()
    };
    let sub_transactions: Vec<_> = ynab_scheduled_transactions
        .scheduled_transactions
        .clone()
        .into_iter()
        .flat_map(|st| st.subtransactions)
        .collect();
    let ynab_categories = fake::vec![Category; sub_transactions.len()];
    ynab_scheduled_transactions
        .scheduled_transactions
        .iter_mut()
        .for_each(|st| {
            st.subtransactions.iter_mut().for_each(|sub| {
                sub.category_id = ynab_categories
                    .clone()
                    .into_iter()
                    .map(|cat| cat.id)
                    .choose(&mut rand::thread_rng());
            });
        });

    check_get(
        pool,
        0,
        YnabData(ynab_scheduled_transactions),
        Some(DbData(ynab_categories)),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn calls_ynab_api_when_category_is_not_in_db(pool: SqlitePool) {
    let ynab_scheduled_transactions = ScheduledTransactionsDetailDelta {
        scheduled_transactions: fake::vec![ScheduledTransactionDetail; 1..10],
        ..Faker.fake()
    };
    let ynab_calls = count_sub_transaction_ids(&ynab_scheduled_transactions);

    check_get(
        pool,
        ynab_calls,
        YnabData(ynab_scheduled_transactions),
        None,
    )
    .await;
}
