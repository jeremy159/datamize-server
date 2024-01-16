use chrono::{Datelike, Days, Local, Months};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use uuid::Uuid;
use ynab::RecurFrequency;

use crate::{
    Budgeter, BudgeterConfig, BudgeterExt, Configured, DatamizeScheduledTransaction, TotalBudgeter,
};

#[test]
fn salary_is_0_when_no_scheduled_transactions() {
    let config: BudgeterConfig = Faker.fake();
    let budgeter: Budgeter<Configured> = config.clone().into();
    let budgeter = budgeter.compute_salary(&[]);

    assert_eq!(budgeter.salary(), 0);
    assert_eq!(budgeter.salary_month(), 0);
}

#[test]
fn salary_is_0_when_no_linked_scheduled_transactions() {
    let config: BudgeterConfig = Faker.fake();
    let budgeter: Budgeter<Configured> = config.clone().into();
    let budgeter = budgeter.compute_salary(&fake::vec![DatamizeScheduledTransaction; 1..5]);

    assert_eq!(budgeter.salary(), 0);
    assert_eq!(budgeter.salary_month(), 0);
}

#[test]
fn salary_is_linked_scheduled_transactions_when_not_repeating() {
    let config = BudgeterConfig {
        payee_ids: fake::vec![Uuid; 1..3],
        ..Faker.fake()
    };
    let budgeter: Budgeter<Configured> = config.clone().into();
    let transaction = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: Some(RecurFrequency::Never),
        date_first: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        ..Faker.fake()
    };
    let budgeter = budgeter.compute_salary(&vec![transaction.clone()]);

    assert_eq!(budgeter.salary(), transaction.amount);
    assert_eq!(budgeter.salary_month(), transaction.amount);
}

#[test]
fn salary_month_is_twice_linked_scheduled_transactions() {
    let config = BudgeterConfig {
        payee_ids: fake::vec![Uuid; 1..3],
        ..Faker.fake()
    };
    let budgeter: Budgeter<Configured> = config.clone().into();
    let transaction = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: Some(RecurFrequency::TwiceAMonth),
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        date_first: Local::now()
            .date_naive()
            .checked_sub_days(Days::new(1))
            .unwrap(),
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(10))
            .unwrap(),
        ..Faker.fake()
    };
    let budgeter = budgeter.compute_salary(&vec![transaction.clone()]);

    assert_eq!(budgeter.salary(), transaction.amount);
    assert_eq!(budgeter.salary_month(), transaction.amount * 2);
}

#[test]
fn salary_takes_all_linked_scheduled_transactions() {
    let config = BudgeterConfig {
        payee_ids: fake::vec![Uuid; 2..3],
        ..Faker.fake()
    };
    let budgeter: Budgeter<Configured> = config.clone().into();
    let first_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: None,
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        ..Faker.fake()
    };
    let sec_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: None,
        payee_id: Some(budgeter.payee_ids()[1]),
        payee_name: Some(Faker.fake()),
        ..Faker.fake()
    };
    let budgeter = budgeter.compute_salary(&vec![first_trans.clone(), sec_trans.clone()]);

    assert_eq!(budgeter.salary(), first_trans.amount + sec_trans.amount);
    assert_eq!(
        budgeter.salary_month(),
        first_trans.amount + sec_trans.amount
    );
}

#[test]
fn salary_takes_all_linked_scheduled_transactions_even_when_repeated() {
    let config = BudgeterConfig {
        payee_ids: fake::vec![Uuid; 2..3],
        ..Faker.fake()
    };
    let budgeter: Budgeter<Configured> = config.clone().into();
    let first_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: Some(RecurFrequency::TwiceAMonth),
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        date_first: Local::now()
            .date_naive()
            .checked_sub_days(Days::new(1))
            .unwrap(),
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(10))
            .unwrap(),
        ..Faker.fake()
    };
    let sec_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: None,
        payee_id: Some(budgeter.payee_ids()[1]),
        payee_name: Some(Faker.fake()),
        ..Faker.fake()
    };
    let budgeter = budgeter.compute_salary(&vec![first_trans.clone(), sec_trans.clone()]);

    assert_eq!(budgeter.salary(), first_trans.amount + sec_trans.amount);
    assert_eq!(
        budgeter.salary_month(),
        first_trans.amount * 2 + sec_trans.amount
    );
}

#[test]
fn salary_repeats_5_times_when_frequency_is_every_week_on_first_day_of_month() {
    let config = BudgeterConfig {
        payee_ids: fake::vec![Uuid; 2..3],
        ..Faker.fake()
    };
    let budgeter: Budgeter<Configured> = config.clone().into();
    let date_first = Local::now().date_naive().with_day(1).unwrap();
    let date_first = if date_first.month0() == 1 {
        date_first.checked_sub_months(Months::new(1)).unwrap()
    } else {
        date_first
    };
    let transaction = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: Some(RecurFrequency::Weekly),
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        date_first,
        date_next: date_first.checked_add_days(Days::new(7)).unwrap(),
        ..Faker.fake()
    };
    let budgeter = budgeter.compute_salary(&vec![transaction.clone()]);

    assert_eq!(budgeter.salary(), transaction.amount);
    assert_eq!(budgeter.salary_month(), transaction.amount * 5);
}

#[test]
fn total_salary_is_0_when_no_budgeters() {
    let total_budgeter = TotalBudgeter::new();
    let total_budgeter = total_budgeter.compute_salary(&[]);

    assert_eq!(total_budgeter.salary(), 0);
    assert_eq!(total_budgeter.salary_month(), 0);
}

#[test]
fn total_salary_is_sum_of_all_budgeters() {
    let configs = vec![
        BudgeterConfig {
            payee_ids: fake::vec![Uuid; 1..3],
            ..Faker.fake()
        },
        BudgeterConfig {
            payee_ids: fake::vec![Uuid; 1..3],
            ..Faker.fake()
        },
    ];
    let budgeter1: Budgeter<Configured> = configs[0].clone().into();
    let date_first = Local::now().date_naive().with_day(1).unwrap();
    let date_first = if date_first.month0() == 1 {
        date_first.checked_sub_months(Months::new(1)).unwrap()
    } else {
        date_first
    };
    let transaction = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        payee_id: Some(budgeter1.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        date_first,
        date_next: date_first.checked_add_days(Days::new(7)).unwrap(),
        ..Faker.fake()
    };
    let budgeter1 = budgeter1.compute_salary(&vec![transaction.clone()]);
    let budgeter2: Budgeter<Configured> = configs[0].clone().into();
    let date_first = Local::now().date_naive().with_day(1).unwrap();
    let date_first = if date_first.month0() == 1 {
        date_first.checked_sub_months(Months::new(1)).unwrap()
    } else {
        date_first
    };
    let transaction = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        payee_id: Some(budgeter2.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        date_first,
        date_next: date_first.checked_add_days(Days::new(7)).unwrap(),
        ..Faker.fake()
    };
    let budgeter2 = budgeter2.compute_salary(&vec![transaction.clone()]);

    let total_budgeter = TotalBudgeter::new();
    let total_budgeter = total_budgeter.compute_salary(&[budgeter1.clone(), budgeter2.clone()]);

    assert_eq!(
        total_budgeter.salary(),
        budgeter1.salary() + budgeter2.salary()
    );
    assert_eq!(
        total_budgeter.salary_month(),
        budgeter1.salary_month() + budgeter2.salary_month()
    );
}
