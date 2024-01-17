use crate::{
    models::budget_template::{
        expense,
        tests::budgeter::testutils::{
            setup_budgeters_with_salary, setup_budgeters_with_salary_with_name,
            setup_computed_expenses, setup_computed_expenses_with_first_non_external,
        },
    },
    Budgeter, BudgeterExt, ComputedExpenses, ComputedSalary, Expense, TotalBudgeter,
};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;

#[derive(Debug, Clone)]
struct Expected {
    individual_expenses_len: usize,
    individual_expenses: i64,
    common_expenses: i64,
    left_over: i64,
}

#[track_caller]
fn check_method_total_budgeter(
    budgeters: &[Budgeter<ComputedSalary>],
    expenses: &[Expense<expense::Computed>],
    Expected {
        individual_expenses_len,
        individual_expenses: individual_expenses_total,
        common_expenses,
        left_over,
    }: Expected,
) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!(
        "check_method_total_budgeter called from line: {}",
        caller_line_number
    );

    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(budgeters)
        .compute_expenses(expenses, budgeters);

    assert_eq!(individual_expenses.len(), individual_expenses_len);
    assert_eq!(total_budgeter.common_expenses(), common_expenses);
    assert_eq!(
        total_budgeter.individual_expenses(),
        individual_expenses_total
    );
    assert_eq!(total_budgeter.left_over(), left_over);
}

#[track_caller]
fn check_method_budgeter(
    budgeter: Budgeter<ComputedSalary>,
    budgeters: &[Budgeter<ComputedSalary>],
    expenses: &[Expense<expense::Computed>],
) -> (Budgeter<ComputedExpenses>, Vec<Expense<expense::Computed>>) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!(
        "check_method_budgeter called from line: {}",
        caller_line_number
    );

    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(budgeters)
        .compute_expenses(expenses, budgeters);

    let budgeter = budgeter.compute_expenses(&total_budgeter, &individual_expenses);

    (budgeter, individual_expenses.into_iter().cloned().collect())
}

#[test]
fn total_is_0_when_no_budgeters_or_expenses() {
    check_method_total_budgeter(
        &[],
        &[],
        Expected {
            individual_expenses_len: 0,
            individual_expenses: 0,
            common_expenses: 0,
            left_over: 0,
        },
    );
}

#[test]
fn total_left_over_is_salary_when_no_expenses() {
    let budgeters = setup_budgeters_with_salary();
    check_method_total_budgeter(
        &budgeters,
        &[],
        Expected {
            individual_expenses_len: 0,
            individual_expenses: 0,
            common_expenses: 0,
            left_over: budgeters.iter().map(|b| b.salary_month()).sum(),
        },
    );
}

#[test]
fn total_left_over_is_inverse_of_all_expenses_when_no_budgeters() {
    let expenses = setup_computed_expenses();
    let total_expense: i64 = expenses
        .iter()
        .filter(|&e| !e.is_external())
        .map(|e| e.projected_amount())
        .sum();

    check_method_total_budgeter(
        &[],
        &expenses,
        Expected {
            individual_expenses_len: 0,
            individual_expenses: 0,
            common_expenses: total_expense,
            left_over: -total_expense,
        },
    );
}

#[test]
fn total_left_over_is_salary_minus_all_expenses_when_no_budgeters_match() {
    let expenses = setup_computed_expenses();
    let budgeters = setup_budgeters_with_salary();
    let total_expense: i64 = expenses
        .iter()
        .filter(|&e| !e.is_external())
        .map(|e| e.projected_amount())
        .sum();

    check_method_total_budgeter(
        &budgeters,
        &expenses,
        Expected {
            individual_expenses_len: 0,
            individual_expenses: 0,
            common_expenses: total_expense,
            left_over: budgeters.iter().map(|b| b.salary_month()).sum::<i64>() - total_expense,
        },
    );
}

#[test]
fn total_left_over_is_salary_minus_all_expenses_even_when_budgeters_match() {
    let expenses = setup_computed_expenses_with_first_non_external();
    let budgeters = setup_budgeters_with_salary_with_name(expenses[0].name());
    let total_expense: i64 = expenses
        .iter()
        .filter(|&e| !e.is_external())
        .map(|e| e.projected_amount())
        .sum();

    check_method_total_budgeter(
        &budgeters,
        &expenses,
        Expected {
            individual_expenses_len: 1,
            individual_expenses: expenses[0].projected_amount(),
            common_expenses: total_expense - expenses[0].projected_amount(),
            left_over: budgeters.iter().map(|b| b.salary_month()).sum::<i64>() - total_expense,
        },
    );
}

#[test]
fn proportion_is_0_when_no_total_salary() {
    let expenses = setup_computed_expenses();
    let budgeter: Budgeter<ComputedSalary> = Faker.fake();
    let (budgeter, _) = check_method_budgeter(budgeter, &[], &expenses);

    assert_eq!(budgeter.proportion(), 0.0);
    assert_eq!(budgeter.common_expenses(), 0);
    assert_eq!(budgeter.individual_expenses(), 0);
    assert_eq!(budgeter.left_over(), budgeter.salary_month());
}

#[test]
fn left_over_is_salary_when_no_expenses() {
    let budgeters = setup_budgeters_with_salary();
    let (budgeter, individual_expenses) =
        check_method_budgeter(budgeters[0].clone(), &budgeters, &[]);

    assert!(individual_expenses.is_empty());
    assert_eq!(budgeter.common_expenses(), 0);
    assert_eq!(budgeter.individual_expenses(), 0);
    assert_eq!(budgeter.left_over(), budgeter.salary_month());
}

#[test]
fn left_over_is_salary_minus_all_expenses_when_no_budgeters_match() {
    let expenses = setup_computed_expenses();
    let budgeters = setup_budgeters_with_salary();
    let (budgeter, individual_expenses) =
        check_method_budgeter(budgeters[0].clone(), &budgeters, &expenses);

    let total_expense: i64 = expenses
        .iter()
        .filter(|&e| !e.is_external())
        .map(|e| e.projected_amount())
        .sum();

    let common_expenses = (budgeter.proportion() * total_expense as f64) as i64;

    assert!(individual_expenses.is_empty());
    assert_eq!(budgeter.common_expenses(), common_expenses);
    assert_eq!(budgeter.individual_expenses(), 0);
    assert_eq!(
        budgeter.left_over(),
        budgeter.salary_month() - common_expenses
    );
}

#[test]
fn left_over_is_salary_minus_common_expenses_proportionally_minus_individual_expenses_when_budgeter_match(
) {
    let expenses = setup_computed_expenses_with_first_non_external();
    let budgeters = setup_budgeters_with_salary_with_name(expenses[0].name());
    let (budgeter, individual_expenses) =
        check_method_budgeter(budgeters[0].clone(), &budgeters, &expenses);

    let total_expense = expenses
        .iter()
        .filter(|&e| !e.is_external())
        .map(|e| e.projected_amount())
        .sum::<i64>()
        - expenses[0].projected_amount();

    let common_expenses = (budgeter.proportion() * total_expense as f64) as i64;

    assert_eq!(individual_expenses.len(), 1);
    assert_eq!(budgeter.common_expenses(), common_expenses);
    assert_eq!(
        budgeter.individual_expenses(),
        individual_expenses
            .into_iter()
            .map(|e| e.projected_amount())
            .sum::<i64>()
    );
    assert_eq!(
        budgeter.left_over(),
        budgeter.salary_month() - common_expenses - budgeter.individual_expenses()
    );
}
