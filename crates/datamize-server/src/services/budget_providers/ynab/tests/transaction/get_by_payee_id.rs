use datamize_domain::Uuid;
use db_sqlite::budget_providers::ynab::sabotage_transactions_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use ynab::{BaseTransactionDetail, TransactionDetail};

use crate::services::{
    budget_providers::ynab::tests::transaction::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

async fn check_get_by_payee_id(
    pool: SqlitePool,
    transactions: &[TransactionDetail],
    payee_id: Uuid,
    expected_resp: Option<Vec<TransactionDetail>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool, Faker.fake()).await;
    if expected_err.is_none() {
        context.set_transactions(transactions).await;
    }
    let service = context.into_service();

    let response = service.get_transactions_by_payee_id(payee_id).await;

    if let Some(mut expected_resp) = expected_resp {
        let mut res_body = response.unwrap();
        res_body.sort_by_key(|t| t.base.amount);
        expected_resp.sort_by_key(|t| t.base.amount);
        assert_eq!(res_body, expected_resp);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_empty_list_when_nothing_in_db(pool: SqlitePool) {
    check_get_by_payee_id(pool, &[], Faker.fake(), Some(vec![]), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_empty_list_when_nothing_linked(pool: SqlitePool) {
    check_get_by_payee_id(
        pool,
        &fake::vec![TransactionDetail; 1..5],
        Faker.fake(),
        Some(vec![]),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_only_what_is_linked(pool: SqlitePool) {
    let payee_id = Faker.fake();
    let trans1 = TransactionDetail {
        base: BaseTransactionDetail {
            payee_id: Some(payee_id),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let trans2 = TransactionDetail {
        base: BaseTransactionDetail {
            payee_id: Some(payee_id),
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let mut fake_trans = fake::vec![TransactionDetail; 1..5];
    fake_trans.push(trans1.clone());
    fake_trans.push(trans2.clone());

    check_get_by_payee_id(
        pool,
        &fake_trans,
        payee_id,
        Some(vec![trans1, trans2]),
        None,
    )
    .await;
}

// FIXME: For some reasons sometimes the test fails... Might be related to the redis test mock
// #[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn issue_with_db_should_not_return_data(pool: SqlitePool) {
    sabotage_transactions_table(&pool).await.unwrap();

    check_get_by_payee_id(
        pool,
        &fake::vec![TransactionDetail; 1..5],
        Faker.fake(),
        None,
        Some(ErrorType::Database),
    )
    .await;
}
