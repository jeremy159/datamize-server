use chrono::{Datelike, Days, Local, Months};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use uuid::Uuid;
use ynab::RecurFrequency;

use crate::{Budgeter, BudgeterConfig, BudgeterExt, Configured, DatamizeScheduledTransaction};

#[derive(Debug, Clone)]
struct Expected {
    salary: i64,
    salary_month: i64,
}

#[track_caller]
fn check_method_budgeter(
    budgeter: Budgeter<Configured>,
    scheduled_transactions: &[DatamizeScheduledTransaction],
    Expected {
        salary,
        salary_month,
    }: Expected,
) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!(
        "check_method_budgeter called from line: {}",
        caller_line_number
    );

    let budgeter = budgeter.compute_salary(scheduled_transactions, &Local::now());
    assert_eq!(budgeter.salary(), salary);
    assert_eq!(budgeter.salary_month(), salary_month);
}

#[test]
fn salary_is_0_when_no_scheduled_transactions() {
    let config: BudgeterConfig = Faker.fake();
    let budgeter: Budgeter<Configured> = config.clone().into();
    check_method_budgeter(
        budgeter,
        &[],
        Expected {
            salary: 0,
            salary_month: 0,
        },
    );
}

#[test]
fn salary_is_0_when_no_linked_scheduled_transactions() {
    let config: BudgeterConfig = Faker.fake();
    let budgeter: Budgeter<Configured> = config.clone().into();
    check_method_budgeter(
        budgeter,
        &fake::vec![DatamizeScheduledTransaction; 1..5],
        Expected {
            salary: 0,
            salary_month: 0,
        },
    );
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
    check_method_budgeter(
        budgeter,
        &vec![transaction.clone()],
        Expected {
            salary: transaction.amount,
            salary_month: transaction.amount,
        },
    );
}

#[test]
fn salary_month_is_twice_linked_scheduled_transactions() {
    let config = BudgeterConfig {
        payee_ids: fake::vec![Uuid; 1..3],
        ..Faker.fake()
    };
    let budgeter: Budgeter<Configured> = config.clone().into();
    let date_first = Local::now().date_naive().with_day(5).unwrap();
    let transaction = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: Some(RecurFrequency::EveryOtherWeek),
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        date_first,
        date_next: date_first.checked_add_days(Days::new(14)).unwrap(),
        ..Faker.fake()
    };
    check_method_budgeter(
        budgeter,
        &vec![transaction.clone()],
        Expected {
            salary: transaction.amount,
            salary_month: transaction.amount * 2,
        },
    );
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
    check_method_budgeter(
        budgeter,
        &vec![first_trans.clone(), sec_trans.clone()],
        Expected {
            salary: first_trans.amount + sec_trans.amount,
            salary_month: first_trans.amount + sec_trans.amount,
        },
    );
}

#[test]
fn salary_takes_all_scheduled_transactions_of_same_payee() {
    let payee_id = Faker.fake();
    let payee_name: String = Faker.fake();
    let config = BudgeterConfig {
        payee_ids: vec![payee_id],
        ..Faker.fake()
    };

    let budgeter: Budgeter<Configured> = config.clone().into();
    let first_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: Some(RecurFrequency::Monthly),
        payee_id: Some(payee_id),
        payee_name: Some(payee_name.clone()),
        date_first: Local::now().date_naive().with_day(15).unwrap(),
        ..Faker.fake()
    };
    let sec_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: Some(RecurFrequency::Monthly),
        payee_id: Some(payee_id),
        payee_name: Some(payee_name),
        date_first: Local::now().date_naive().with_day(28).unwrap(),
        ..Faker.fake()
    };
    check_method_budgeter(
        budgeter,
        &vec![first_trans.clone(), sec_trans.clone()],
        Expected {
            salary: first_trans.amount + sec_trans.amount,
            salary_month: first_trans.amount + sec_trans.amount,
        },
    );
}

#[test]
fn salary_takes_all_linked_scheduled_transactions_even_when_repeated() {
    let config = BudgeterConfig {
        payee_ids: fake::vec![Uuid; 2..3],
        ..Faker.fake()
    };
    let budgeter: Budgeter<Configured> = config.clone().into();
    let date_first = Local::now().date_naive().with_day(5).unwrap();
    let first_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: Some(RecurFrequency::EveryOtherWeek),
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        date_first,
        date_next: date_first.checked_add_days(Days::new(14)).unwrap(),
        ..Faker.fake()
    };
    let sec_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: None,
        payee_id: Some(budgeter.payee_ids()[1]),
        payee_name: Some(Faker.fake()),
        ..Faker.fake()
    };
    check_method_budgeter(
        budgeter,
        &vec![first_trans.clone(), sec_trans.clone()],
        Expected {
            salary: first_trans.amount + sec_trans.amount,
            salary_month: first_trans.amount * 2 + sec_trans.amount,
        },
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
    check_method_budgeter(
        budgeter,
        &vec![transaction.clone()],
        Expected {
            salary: transaction.amount,
            salary_month: transaction.amount * 5,
        },
    );
}
