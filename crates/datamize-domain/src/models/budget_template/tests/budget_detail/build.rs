use std::vec;

use chrono::{DateTime, Local};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use uuid::Uuid;
use ynab::Category;

use crate::{
    BudgetDetails, Budgeter, BudgeterExt, ComputedSalary, DatamizeScheduledTransaction,
    ExpenseCategorization, ExpenseType, SubExpenseType,
};

#[derive(Debug, Clone)]
struct Expected {
    total_monthly_income: i64,
    expenses: Vec<Uuid>,
}

#[track_caller]
fn check_method(
    categories: Vec<Category>,
    scheduled_transactions: Vec<DatamizeScheduledTransaction>,
    date: &DateTime<Local>,
    expenses_categorization: Vec<ExpenseCategorization>,
    budgeters: &[Budgeter<ComputedSalary>],
    Expected {
        total_monthly_income,
        expenses,
    }: Expected,
) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!("check_method called from line: {}", caller_line_number);

    let details = BudgetDetails::build(
        categories,
        scheduled_transactions,
        date,
        expenses_categorization,
        budgeters,
    );

    assert_eq!(
        details.global_metadata().total_monthly_income,
        total_monthly_income,
        "total_monthly_income is not same as expected"
    );

    assert_eq!(
        details.expenses().len(),
        expenses.len(),
        "expenses is not same length as expected"
    );

    let expense_ids = details
        .expenses()
        .iter()
        .map(|e| e.id())
        .collect::<Vec<_>>();

    assert_eq!(expense_ids, expenses, "expenses are not same as expected");
}

#[test]
fn empty_when_no_categories() {
    check_method(
        vec![],
        vec![],
        &Local::now(),
        vec![],
        &[],
        Expected {
            total_monthly_income: 0,
            expenses: vec![],
        },
    );
}

#[test]
fn empty_when_no_expense_categorization() {
    let budgeters = fake::vec![Budgeter<ComputedSalary>; 1..3];

    check_method(
        fake::vec![Category; 3..5],
        fake::vec![DatamizeScheduledTransaction; 3..5],
        &Local::now(),
        vec![],
        &budgeters,
        Expected {
            total_monthly_income: budgeters.iter().map(|b| b.salary_month()).sum(),
            expenses: vec![],
        },
    );
}

#[test]
fn empty_when_all_categories_are_hidden_or_deleted() {
    let budgeters = fake::vec![Budgeter<ComputedSalary>; 1..3];
    let categories = vec![
        Category {
            deleted: true,
            hidden: false,
            ..Faker.fake()
        },
        Category {
            deleted: false,
            hidden: true,
            ..Faker.fake()
        },
        Category {
            deleted: true,
            hidden: true,
            ..Faker.fake()
        },
    ];

    check_method(
        categories,
        fake::vec![DatamizeScheduledTransaction; 3..5],
        &Local::now(),
        fake::vec![ExpenseCategorization; 1..3],
        &budgeters,
        Expected {
            total_monthly_income: budgeters.iter().map(|b| b.salary_month()).sum(),
            expenses: vec![],
        },
    );
}

#[test]
fn only_expenses_with_categorization() {
    let budgeters = fake::vec![Budgeter<ComputedSalary>; 1..3];
    let category_group_id = Faker.fake();
    let category_group_id2 = Faker.fake();
    let categories = vec![
        Category {
            deleted: false,
            hidden: false,
            category_group_id: category_group_id2,
            ..Faker.fake()
        },
        Category {
            deleted: false,
            hidden: false,
            category_group_id,
            ..Faker.fake()
        },
        Category {
            deleted: false,
            hidden: false,
            ..Faker.fake()
        },
        Category {
            deleted: false,
            hidden: false,
            ..Faker.fake()
        },
    ];

    let expenses_categorization = vec![
        ExpenseCategorization {
            id: category_group_id,
            expense_type: ExpenseType::Fixed,
            sub_expense_type: SubExpenseType::Housing,
            ..Faker.fake()
        },
        ExpenseCategorization {
            id: category_group_id2,
            expense_type: ExpenseType::Fixed,
            sub_expense_type: SubExpenseType::Transport,
            ..Faker.fake()
        },
    ];
    let expenses = vec![categories[1].id, categories[0].id];

    check_method(
        categories,
        fake::vec![DatamizeScheduledTransaction; 3..5],
        &Local::now(),
        expenses_categorization,
        &budgeters,
        Expected {
            total_monthly_income: budgeters.iter().map(|b| b.salary_month()).sum(),
            expenses,
        },
    );
}
