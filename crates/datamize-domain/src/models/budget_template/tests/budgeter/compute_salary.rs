use chrono::{Datelike, Days, Local, Months};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use uuid::Uuid;
use ynab::{RecurFrequency, ScheduledSubTransaction};

use crate::{Budgeter, BudgeterConfig, BudgeterExt, Configured, DatamizeScheduledTransaction};

#[derive(Debug, Clone)]
struct Expected {
    salary_month: i64,
}

#[track_caller]
fn check_method_budgeter(
    budgeter: Budgeter<Configured>,
    scheduled_transactions: &[DatamizeScheduledTransaction],
    inflow_cat_id: Option<Uuid>,
    Expected { salary_month }: Expected,
) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!(
        "check_method_budgeter called from line: {}",
        caller_line_number
    );

    let budgeter = budgeter.compute_salary(scheduled_transactions, &Local::now(), inflow_cat_id);
    assert_eq!(budgeter.salary_month(), salary_month);
}

#[test]
fn salary_is_0_when_no_scheduled_transactions() {
    let config: BudgeterConfig = Faker.fake();
    let budgeter: Budgeter<Configured> = config.clone().into();
    check_method_budgeter(budgeter, &[], None, Expected { salary_month: 0 });
}

#[test]
fn salary_is_0_when_no_linked_scheduled_transactions() {
    let config: BudgeterConfig = Faker.fake();
    let budgeter: Budgeter<Configured> = config.clone().into();
    check_method_budgeter(
        budgeter,
        &fake::vec![DatamizeScheduledTransaction; 1..5],
        None,
        Expected { salary_month: 0 },
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
        frequency: RecurFrequency::Never,
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
        None,
        Expected {
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
        frequency: RecurFrequency::EveryOtherWeek,
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        date_first,
        date_next: date_first.checked_add_days(Days::new(14)).unwrap(),
        ..Faker.fake()
    };
    check_method_budgeter(
        budgeter,
        &vec![transaction.clone()],
        None,
        Expected {
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
        frequency: RecurFrequency::Never,
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        ..Faker.fake()
    };
    let sec_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: RecurFrequency::Never,
        payee_id: Some(budgeter.payee_ids()[1]),
        payee_name: Some(Faker.fake()),
        ..Faker.fake()
    };
    check_method_budgeter(
        budgeter,
        &vec![first_trans.clone(), sec_trans.clone()],
        None,
        Expected {
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
        frequency: RecurFrequency::Monthly,
        payee_id: Some(payee_id),
        payee_name: Some(payee_name.clone()),
        date_first: Local::now().date_naive().with_day(15).unwrap(),
        ..Faker.fake()
    };
    let sec_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: RecurFrequency::Monthly,
        payee_id: Some(payee_id),
        payee_name: Some(payee_name),
        date_first: Local::now().date_naive().with_day(28).unwrap(),
        ..Faker.fake()
    };
    check_method_budgeter(
        budgeter,
        &vec![first_trans.clone(), sec_trans.clone()],
        None,
        Expected {
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
        frequency: RecurFrequency::EveryOtherWeek,
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        date_first,
        date_next: date_first.checked_add_days(Days::new(14)).unwrap(),
        ..Faker.fake()
    };
    let sec_trans = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: RecurFrequency::Never,
        payee_id: Some(budgeter.payee_ids()[1]),
        payee_name: Some(Faker.fake()),
        ..Faker.fake()
    };
    check_method_budgeter(
        budgeter,
        &vec![first_trans.clone(), sec_trans.clone()],
        None,
        Expected {
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
        frequency: RecurFrequency::Weekly,
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        date_first,
        date_next: date_first.checked_add_days(Days::new(7)).unwrap(),
        ..Faker.fake()
    };
    check_method_budgeter(
        budgeter,
        &vec![transaction.clone()],
        None,
        Expected {
            salary_month: transaction.amount * 5,
        },
    );
}

#[test]
fn salary_takes_inflow_from_sub_transaction() {
    let config = BudgeterConfig {
        payee_ids: fake::vec![Uuid; 2..3],
        ..Faker.fake()
    };
    let budgeter: Budgeter<Configured> = config.clone().into();
    let inflow_cat_id = Faker.fake();
    let amount = (1..100000).fake();
    let inflow_amount = amount + 50000;

    let subtransactions = vec![
        ScheduledSubTransaction {
            category_id: Some(inflow_cat_id),
            amount: inflow_amount,
            ..Faker.fake()
        },
        ScheduledSubTransaction { ..Faker.fake() },
    ];
    let transaction = DatamizeScheduledTransaction {
        amount,
        frequency: RecurFrequency::Never,
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        subtransactions,
        ..Faker.fake()
    };

    check_method_budgeter(
        budgeter,
        &vec![transaction.clone()],
        Some(inflow_cat_id),
        Expected {
            salary_month: inflow_amount,
        },
    );
}

#[test]
fn salary_does_not_take_inflow_from_sub_transaction_even_if_defined() {
    let config = BudgeterConfig {
        payee_ids: fake::vec![Uuid; 2..3],
        ..Faker.fake()
    };
    let budgeter: Budgeter<Configured> = config.clone().into();
    let inflow_cat_id = Faker.fake();

    let subtransactions = fake::vec![ScheduledSubTransaction; 2..3];
    let transaction = DatamizeScheduledTransaction {
        amount: (1..100000).fake(),
        frequency: RecurFrequency::Never,
        payee_id: Some(budgeter.payee_ids()[0]),
        payee_name: Some(Faker.fake()),
        subtransactions,
        ..Faker.fake()
    };

    check_method_budgeter(
        budgeter,
        &vec![transaction.clone()],
        Some(inflow_cat_id),
        Expected {
            salary_month: transaction.amount,
        },
    );
}
