use datamize_domain::{Incomes, SavingRate, Savings, Uuid};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use ynab::{BaseTransactionDetail, TransactionDetail};

use crate::services::balance_sheet::tests::saving_rate::testutils::TestContext;

async fn check_get(
    pool: SqlitePool,
    transactions: &[TransactionDetail],
    saving_rate: &SavingRate,
    mut expected_resp: Vec<TransactionDetail>,
) {
    let context = TestContext::setup(pool).await;
    context.set_transactions(transactions).await;
    let service = context.into_service();

    let actual = service.get_transactions_for(saving_rate).await;

    expected_resp.sort_by_key(|t| t.base.amount);

    assert_eq!(actual, expected_resp);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_when_no_categories_or_payees(pool: SqlitePool) {
    let saving_rate = SavingRate {
        savings: Savings {
            category_ids: vec![],
            ..Faker.fake()
        },
        incomes: Incomes {
            payee_ids: vec![],
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    check_get(
        pool,
        &fake::vec![TransactionDetail; 1..5],
        &saving_rate,
        vec![],
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn empty_when_no_categories_or_payees_linked(pool: SqlitePool) {
    let saving_rate = SavingRate {
        savings: Savings {
            category_ids: fake::vec![Uuid; 2],
            ..Faker.fake()
        },
        incomes: Incomes {
            payee_ids: fake::vec![Uuid; 2],
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    check_get(
        pool,
        &fake::vec![TransactionDetail; 1..5],
        &saving_rate,
        vec![],
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_multiple_transactions_of_linked_categories_only(pool: SqlitePool) {
    let cat_id1 = Faker.fake();
    let cat_id2 = Faker.fake();
    let saving_rate = SavingRate {
        savings: Savings {
            category_ids: vec![cat_id1, cat_id2],
            ..Faker.fake()
        },
        incomes: Incomes {
            payee_ids: fake::vec![Uuid; 2],
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let trans1 = TransactionDetail {
        base: BaseTransactionDetail {
            category_id: Some(cat_id1),
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let trans2 = TransactionDetail {
        base: BaseTransactionDetail {
            category_id: Some(cat_id2),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let mut fake_trans = fake::vec![TransactionDetail; 1..5];
    fake_trans.push(trans1.clone());
    fake_trans.push(trans2.clone());

    check_get(pool, &fake_trans, &saving_rate, vec![trans1, trans2]).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_transactions_of_same_linked_categories_only(pool: SqlitePool) {
    let cat_id1 = Faker.fake();
    let cat_id2 = Faker.fake();
    let saving_rate = SavingRate {
        savings: Savings {
            category_ids: vec![cat_id1, cat_id2, Faker.fake()],
            ..Faker.fake()
        },
        incomes: Incomes {
            payee_ids: fake::vec![Uuid; 2],
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let trans1 = TransactionDetail {
        base: BaseTransactionDetail {
            category_id: Some(cat_id1),
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let trans2 = TransactionDetail {
        base: BaseTransactionDetail {
            category_id: Some(cat_id1),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let mut fake_trans = fake::vec![TransactionDetail; 1..5];
    fake_trans.push(trans1.clone());
    fake_trans.push(trans2.clone());

    check_get(pool, &fake_trans, &saving_rate, vec![trans1, trans2]).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_multiple_transactions_of_linked_payees_only(pool: SqlitePool) {
    let payee_id1 = Faker.fake();
    let payee_id2 = Faker.fake();
    let saving_rate = SavingRate {
        savings: Savings {
            category_ids: fake::vec![Uuid; 2],
            ..Faker.fake()
        },
        incomes: Incomes {
            payee_ids: vec![payee_id1, payee_id2],
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let trans1 = TransactionDetail {
        base: BaseTransactionDetail {
            payee_id: Some(payee_id1),
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let trans2 = TransactionDetail {
        base: BaseTransactionDetail {
            payee_id: Some(payee_id2),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let mut fake_trans = fake::vec![TransactionDetail; 1..5];
    fake_trans.push(trans1.clone());
    fake_trans.push(trans2.clone());

    check_get(pool, &fake_trans, &saving_rate, vec![trans1, trans2]).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_transactions_of_same_linked_payee_only(pool: SqlitePool) {
    let payee_id1 = Faker.fake();
    let payee_id2 = Faker.fake();
    let saving_rate = SavingRate {
        savings: Savings {
            category_ids: fake::vec![Uuid; 2],
            ..Faker.fake()
        },
        incomes: Incomes {
            payee_ids: vec![payee_id1, payee_id2],
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let trans1 = TransactionDetail {
        base: BaseTransactionDetail {
            payee_id: Some(payee_id1),
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let trans2 = TransactionDetail {
        base: BaseTransactionDetail {
            payee_id: Some(payee_id2),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let mut fake_trans = fake::vec![TransactionDetail; 1..5];
    fake_trans.push(trans1.clone());
    fake_trans.push(trans2.clone());

    check_get(pool, &fake_trans, &saving_rate, vec![trans1, trans2]).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn does_not_return_twice_same_transaction(pool: SqlitePool) {
    let payee_id1 = Faker.fake();
    let cat_id1 = Faker.fake();
    let saving_rate = SavingRate {
        savings: Savings {
            category_ids: vec![cat_id1],
            ..Faker.fake()
        },
        incomes: Incomes {
            payee_ids: vec![payee_id1],
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    let trans1 = TransactionDetail {
        base: BaseTransactionDetail {
            payee_id: Some(payee_id1),
            category_id: Some(cat_id1),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let mut fake_trans = fake::vec![TransactionDetail; 1..5];
    fake_trans.push(trans1.clone());

    check_get(pool, &fake_trans, &saving_rate, vec![trans1]).await;
}
