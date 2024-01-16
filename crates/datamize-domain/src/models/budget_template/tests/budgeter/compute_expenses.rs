use crate::{
    models::budget_template::tests::budgeter::testutils::{
        setup_budgeters_with_salary, setup_budgeters_with_salary_with_name,
        setup_computed_expenses, setup_computed_expenses_with_first_non_external,
    },
    Budgeter, BudgeterExt, ComputedSalary, TotalBudgeter,
};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;

#[test]
fn total_is_0_when_no_budgeters_or_expenses() {
    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(&[])
        .compute_expenses(&[], &[]);

    assert!(individual_expenses.is_empty());
    assert_eq!(total_budgeter.common_expenses(), 0);
    assert_eq!(total_budgeter.individual_expenses(), 0);
    assert_eq!(total_budgeter.left_over(), 0);
}

#[test]
fn total_left_over_is_salary_when_no_expenses() {
    let budgeters = setup_budgeters_with_salary();
    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(&budgeters)
        .compute_expenses(&[], &budgeters);

    assert!(individual_expenses.is_empty());
    assert_eq!(total_budgeter.common_expenses(), 0);
    assert_eq!(total_budgeter.individual_expenses(), 0);
    assert_eq!(total_budgeter.left_over(), total_budgeter.salary_month());
}

#[test]
fn total_left_over_is_inverse_of_all_expenses_when_no_budgeters() {
    let expenses = setup_computed_expenses();
    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(&[])
        .compute_expenses(&expenses, &[]);

    let total_expense: i64 = expenses
        .iter()
        .filter(|&e| !e.is_external())
        .map(|e| e.projected_amount())
        .sum();

    assert!(individual_expenses.is_empty());
    assert_eq!(total_budgeter.common_expenses(), total_expense);
    assert_eq!(total_budgeter.individual_expenses(), 0);
    assert_eq!(total_budgeter.left_over(), -total_expense);
}

#[test]
fn total_left_over_is_salary_minus_all_expenses_when_no_budgeters_match() {
    let expenses = setup_computed_expenses();
    let budgeters = setup_budgeters_with_salary();
    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(&budgeters)
        .compute_expenses(&expenses, &budgeters);

    let total_expense: i64 = expenses
        .iter()
        .filter(|&e| !e.is_external())
        .map(|e| e.projected_amount())
        .sum();

    assert!(individual_expenses.is_empty());
    assert_eq!(total_budgeter.common_expenses(), total_expense);
    assert_eq!(total_budgeter.individual_expenses(), 0);
    assert_eq!(
        total_budgeter.left_over(),
        total_budgeter.salary_month() - total_expense
    );
}

#[test]
fn total_left_over_is_salary_minus_all_expenses_even_when_budgeters_match() {
    let expenses = setup_computed_expenses_with_first_non_external();
    let budgeters = setup_budgeters_with_salary_with_name(expenses[0].name());
    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(&budgeters)
        .compute_expenses(&expenses, &budgeters);

    let total_expense: i64 = expenses
        .iter()
        .filter(|&e| !e.is_external())
        .map(|e| e.projected_amount())
        .sum();

    assert_eq!(individual_expenses.len(), 1);
    assert_eq!(
        total_budgeter.common_expenses(),
        total_expense - expenses[0].projected_amount()
    );
    assert_eq!(
        total_budgeter.individual_expenses(),
        expenses[0].projected_amount()
    );
    assert_eq!(
        total_budgeter.left_over(),
        total_budgeter.salary_month() - total_expense
    );
}

#[test]
fn proportion_is_0_when_no_total_salary() {
    let expenses = setup_computed_expenses();
    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(&[])
        .compute_expenses(&expenses, &[]);

    let budgeter: Budgeter<ComputedSalary> = Faker.fake();
    let budgeter = budgeter.compute_expenses(&total_budgeter, &individual_expenses);

    assert_eq!(budgeter.proportion(), 0.0);
    assert_eq!(budgeter.common_expenses(), 0);
    assert_eq!(budgeter.individual_expenses(), 0);
    assert_eq!(budgeter.left_over(), budgeter.salary_month());
}

#[test]
fn left_over_is_salary_when_no_expenses() {
    let budgeters = setup_budgeters_with_salary();
    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(&budgeters)
        .compute_expenses(&[], &budgeters);

    let budgeter = budgeters[0]
        .clone()
        .compute_expenses(&total_budgeter, &individual_expenses);

    assert!(individual_expenses.is_empty());
    assert_eq!(budgeter.common_expenses(), 0);
    assert_eq!(budgeter.individual_expenses(), 0);
    assert_eq!(budgeter.left_over(), budgeter.salary_month());
}

#[test]
fn left_over_is_salary_minus_all_expenses_when_no_budgeters_match() {
    let expenses = setup_computed_expenses();
    let budgeters = setup_budgeters_with_salary();
    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(&budgeters)
        .compute_expenses(&expenses, &budgeters);

    let total_expense: i64 = expenses
        .iter()
        .filter(|&e| !e.is_external())
        .map(|e| e.projected_amount())
        .sum();

    let budgeter = budgeters[0]
        .clone()
        .compute_expenses(&total_budgeter, &individual_expenses);

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
    let total_budgeter = TotalBudgeter::new();
    let (total_budgeter, individual_expenses) = total_budgeter
        .compute_salary(&budgeters)
        .compute_expenses(&expenses, &budgeters);

    let total_expense = expenses
        .iter()
        .filter(|&e| !e.is_external())
        .map(|e| e.projected_amount())
        .sum::<i64>()
        - expenses[0].projected_amount();

    let budgeter = budgeters[0]
        .clone()
        .compute_expenses(&total_budgeter, &individual_expenses);

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
