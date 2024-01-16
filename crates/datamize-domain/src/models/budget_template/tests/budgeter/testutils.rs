use chrono::{Days, Local};
use fake::{Fake, Faker};
use uuid::Uuid;
use ynab::{Category, RecurFrequency};

use crate::{
    models::budget_template::expense, Budgeter, BudgeterConfig, BudgeterExt, ComputedSalary,
    Configured, DatamizeScheduledTransaction, Expense, Uncomputed,
};

pub fn setup_budgeters_with_salary() -> Vec<Budgeter<ComputedSalary>> {
    fake::vec![Budgeter<ComputedSalary>; 2]
}

pub fn setup_budgeters_with_salary_with_name(first_name: &str) -> Vec<Budgeter<ComputedSalary>> {
    let config = BudgeterConfig {
        payee_ids: fake::vec![Uuid; 1..3],
        name: first_name.to_owned(),
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

    vec![budgeter, Faker.fake()]
}

pub fn setup_computed_expenses() -> Vec<Expense<expense::Computed>> {
    fake::vec![Expense<expense::Computed>; 5..10]
}

pub fn setup_computed_expenses_with_first_non_external() -> Vec<Expense<expense::Computed>> {
    let category = Category {
        goal_type: Some(Faker.fake()),
        goal_under_funded: Some(0),
        ..Faker.fake()
    };
    let scheduled_transactions = fake::vec![DatamizeScheduledTransaction; 1..3];

    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense
        .with_scheduled_transactions(scheduled_transactions.clone())
        .compute_amounts()
        .compute_proportions((1..1000).fake());
    let fake_vec = fake::vec![Expense<expense::Computed>; 5..10];
    let mut vec = vec![expense];
    vec.extend(fake_vec);
    vec
}
