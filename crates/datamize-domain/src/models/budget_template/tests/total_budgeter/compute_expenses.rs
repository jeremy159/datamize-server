use crate::{
    models::budget_template::{
        expense,
        tests::total_budgeter::testutils::{
            setup_budgeters_with_salary, setup_budgeters_with_salary_with_name,
            setup_computed_expenses, setup_computed_expenses_with_first_non_external,
        },
    },
    Budgeter, BudgeterExt, ComputedSalary, Expense, TotalBudgeter,
};
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
    let total_expense: i64 = expenses.iter().map(|e| e.projected_amount()).sum();

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
    let total_expense: i64 = expenses.iter().map(|e| e.projected_amount()).sum();

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
    let total_expense: i64 = expenses.iter().map(|e| e.projected_amount()).sum();

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
