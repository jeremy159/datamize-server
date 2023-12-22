use crate::{Incomes, SavingRate, Savings};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use uuid::Uuid;
use ynab::TransactionDetail;

#[test]
fn compute_savings_sets_total_as_extra_balance_when_no_transactions() {
    let mut savings: Savings = Faker.fake();
    let transactions: Vec<TransactionDetail> = vec![];
    savings.compute_total(&transactions);

    assert_eq!(savings.total, savings.extra_balance);
}

#[test]
fn compute_savings_sets_total_as_extra_balance_when_no_transactions_linked() {
    let mut savings: Savings = Faker.fake();
    let transactions: Vec<TransactionDetail> = fake::vec![TransactionDetail; 1..3];
    savings.compute_total(&transactions);

    assert_eq!(savings.total, savings.extra_balance);
}

#[test]
fn compute_savings_sets_total_with_linked_transactions() {
    let mut savings: Savings = Savings {
        category_ids: fake::vec![Uuid; 3],
        ..Faker.fake()
    };
    let mut transactions: Vec<TransactionDetail> = fake::vec![TransactionDetail; 3..5];
    transactions[0].base.category_id = Some(savings.category_ids[0]);
    transactions[1].base.category_id = Some(savings.category_ids[1]);
    savings.compute_total(&transactions);

    assert_ne!(savings.total, savings.extra_balance);
    let transcations_total = transactions[0].base.amount + transactions[1].base.amount;
    assert_eq!(savings.total, savings.extra_balance + transcations_total);
}

#[test]
fn compute_incomes_sets_total_as_extra_balance_when_no_transactions() {
    let mut incomes: Incomes = Faker.fake();
    let transactions: Vec<TransactionDetail> = vec![];
    incomes.compute_total(&transactions);

    assert_eq!(incomes.total, incomes.extra_balance);
}

#[test]
fn compute_incomes_sets_total_as_extra_balance_when_no_transactions_linked() {
    let mut incomes: Incomes = Faker.fake();
    let transactions: Vec<TransactionDetail> = fake::vec![TransactionDetail; 1..3];
    incomes.compute_total(&transactions);

    assert_eq!(incomes.total, incomes.extra_balance);
}

#[test]
fn compute_incomes_sets_total_with_linked_transactions() {
    let mut incomes: Incomes = Incomes {
        payee_ids: fake::vec![Uuid; 3],
        ..Faker.fake()
    };
    let mut transactions: Vec<TransactionDetail> = fake::vec![TransactionDetail; 3..5];
    transactions[0].base.payee_id = Some(incomes.payee_ids[0]);
    transactions[1].base.payee_id = Some(incomes.payee_ids[1]);
    incomes.compute_total(&transactions);

    assert_ne!(incomes.total, incomes.extra_balance);
    let transcations_total = transactions[0].base.amount + transactions[1].base.amount;
    assert_eq!(incomes.total, incomes.extra_balance + transcations_total);
}

#[test]
fn compute_totals_calls_both_savings_and_incomes() {
    let mut saving_rate = SavingRate {
        savings: Savings {
            category_ids: fake::vec![Uuid; 3],
            ..Faker.fake()
        },
        incomes: Incomes {
            payee_ids: fake::vec![Uuid; 3],
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let mut transactions: Vec<TransactionDetail> = fake::vec![TransactionDetail; 3..5];
    transactions[0].base.category_id = Some(saving_rate.savings.category_ids[0]);
    transactions[1].base.payee_id = Some(saving_rate.incomes.payee_ids[0]);
    let savings_total_before = saving_rate.savings.total;
    let incomes_total_before = saving_rate.incomes.total;
    saving_rate.compute_totals(&transactions);

    assert_ne!(savings_total_before, saving_rate.savings.total);
    assert_ne!(incomes_total_before, saving_rate.incomes.total);
}
