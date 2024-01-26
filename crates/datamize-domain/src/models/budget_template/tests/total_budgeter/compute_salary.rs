use chrono::{Datelike, Days, Local, Months};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use uuid::Uuid;

use crate::{
    Budgeter, BudgeterConfig, BudgeterExt, ComputedSalary, Configured,
    DatamizeScheduledTransaction, TotalBudgeter,
};

#[derive(Debug, Clone)]
struct Expected {
    salary: i64,
    salary_month: i64,
}

#[track_caller]
fn check_method_total_budgeter(
    budgeters: &[Budgeter<ComputedSalary>],
    Expected {
        salary,
        salary_month,
    }: Expected,
) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!(
        "check_method_total_budgeter called from line: {}",
        caller_line_number
    );

    let total_budgeter = TotalBudgeter::new();
    let total_budgeter = total_budgeter.compute_salary(budgeters);

    assert_eq!(total_budgeter.salary(), salary);
    assert_eq!(total_budgeter.salary_month(), salary_month);
}

#[test]
fn total_salary_is_0_when_no_budgeters() {
    check_method_total_budgeter(
        &[],
        Expected {
            salary: 0,
            salary_month: 0,
        },
    );
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
    let budgeter1 =
        budgeter1.compute_salary(&vec![transaction.clone()], &Local::now(), Faker.fake());
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
    let budgeter2 =
        budgeter2.compute_salary(&vec![transaction.clone()], &Local::now(), Faker.fake());

    check_method_total_budgeter(
        &[budgeter1.clone(), budgeter2.clone()],
        Expected {
            salary: budgeter1.salary() + budgeter2.salary(),
            salary_month: budgeter1.salary_month() + budgeter2.salary_month(),
        },
    );
}
