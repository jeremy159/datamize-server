use chrono::{Days, Local};
use fake::{Fake, Faker};
use rand::seq::SliceRandom;
use ynab::{Category, GoalType};

use crate::{DatamizeScheduledTransaction, Expense, Uncomputed};

#[test]
fn correctly_adds_scheduled_transactions() {
    let category: Category = Faker.fake();
    let expense: Expense<Uncomputed> = category.clone().into();
    let scheduled_transactions = fake::vec![DatamizeScheduledTransaction; 1..3];

    assert_eq!(expense.scheduled_transactions(), []);
    let expense = expense.with_scheduled_transactions(scheduled_transactions.clone());
    assert_eq!(expense.scheduled_transactions(), &scheduled_transactions);
}

#[test]
fn compute_projected_amount_is_inverted_total_of_scheduled_transactions() {
    let category = Category {
        goal_type: None,
        ..Faker.fake()
    };
    let expense: Expense<Uncomputed> = category.clone().into();
    let scheduled_transactions = fake::vec![DatamizeScheduledTransaction; 1..3];
    let expense = expense
        .with_scheduled_transactions(scheduled_transactions.clone())
        .compute_amounts();

    assert_eq!(
        expense.projected_amount(),
        scheduled_transactions
            .into_iter()
            .map(|st| -st.amount)
            .sum::<i64>()
    );
}

#[test]
fn compute_projected_amount_is_inverted_total_of_scheduled_transactions_when_goal_debt() {
    let category = Category {
        goal_type: Some(GoalType::Debt),
        ..Faker.fake()
    };
    let expense: Expense<Uncomputed> = category.clone().into();
    let scheduled_transactions = fake::vec![DatamizeScheduledTransaction; 1..3];
    let expense = expense
        .with_scheduled_transactions(scheduled_transactions.clone())
        .compute_amounts();

    assert_eq!(
        expense.projected_amount(),
        scheduled_transactions
            .into_iter()
            .map(|st| -st.amount)
            .sum::<i64>()
    );
}

#[test]
fn compute_projected_amount_is_goal_target_and_scheduled_transaction_when_goal_not_debt_or_plan_spending(
) {
    let goals = [
        GoalType::MonthlyFunding,
        GoalType::TargetBalance,
        GoalType::TargetBalanceByDate,
    ];
    let category = Category {
        goal_type: Some(goals.choose(&mut rand::thread_rng()).unwrap().to_owned()),
        ..Faker.fake()
    };
    let expense: Expense<Uncomputed> = category.clone().into();
    let scheduled_transactions = fake::vec![DatamizeScheduledTransaction; 1];
    let expense = expense
        .with_scheduled_transactions(scheduled_transactions.clone())
        .compute_amounts();

    assert_eq!(
        expense.projected_amount(),
        scheduled_transactions
            .into_iter()
            .map(|st| -st.amount)
            .sum::<i64>()
            + category.goal_target
    );
}

#[test]
fn compute_projected_amount_when_goal_target_is_plan_spending_and_cadence_monthly() {
    let mut category = Category {
        goal_type: Some(GoalType::PlanYourSpending),
        goal_cadence: Some(1),
        goal_cadence_frequency: Some((1..13).fake()),
        ..Faker.fake()
    };
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense.compute_amounts();

    assert_eq!(
        expense.projected_amount(),
        category.goal_target / category.goal_cadence_frequency.unwrap() as i64,
        "Is goal target divided by frequency"
    );

    category.goal_cadence_frequency = None;
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense.compute_amounts();

    assert_eq!(
        expense.projected_amount(),
        category.goal_target,
        "Is goal target when no frequency is set"
    );

    category.goal_cadence_frequency = Some(0);
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense.compute_amounts();

    assert_eq!(
        expense.projected_amount(),
        0,
        "Is 0 when frequency is wrongly set"
    );
}

#[test]
fn compute_projected_amount_when_goal_target_is_plan_spending_and_cadence_every_2_years() {
    let category = Category {
        goal_type: Some(GoalType::PlanYourSpending),
        goal_cadence: Some(14),
        goal_cadence_frequency: Faker.fake(),
        ..Faker.fake()
    };
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense.compute_amounts();

    assert_eq!(
        expense.projected_amount(),
        category.goal_target / 24,
        "Is goal target divided by 24 months"
    );
}

#[test]
fn compute_projected_amount_when_goal_target_is_plan_spending_and_cadence_every_x_months() {
    let cadence = (3..=13).fake();

    let category = Category {
        goal_type: Some(GoalType::PlanYourSpending),
        goal_cadence: Some(cadence),
        goal_cadence_frequency: Faker.fake(),
        ..Faker.fake()
    };
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense.compute_amounts();

    assert_eq!(
        expense.projected_amount(),
        category.goal_target / (cadence - 1) as i64,
        "Is goal target divided by cadence minus 1"
    );
}

#[test]
fn compute_current_amount_is_inverted_total_of_scheduled_transactions() {
    let category = Category {
        goal_type: None,
        ..Faker.fake()
    };
    let expense: Expense<Uncomputed> = category.clone().into();
    let mut scheduled_transactions = fake::vec![DatamizeScheduledTransaction; 1];
    scheduled_transactions[0].amount = -category.budgeted;
    let expense = expense
        .with_scheduled_transactions(scheduled_transactions.clone())
        .compute_amounts();

    assert_eq!(
        expense.current_amount(),
        scheduled_transactions
            .into_iter()
            .map(|st| -st.amount)
            .sum::<i64>()
    );
}

#[test]
fn compute_current_amount_when_category_has_goal() {
    let mut category = Category {
        goal_type: Some(Faker.fake()),
        goal_under_funded: Some(0),
        ..Faker.fake()
    };
    let scheduled_transactions = fake::vec![DatamizeScheduledTransaction; 1..3];

    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense
        .with_scheduled_transactions(scheduled_transactions.clone())
        .compute_amounts();

    assert_eq!(
        expense.current_amount(),
        category.budgeted,
        "Is budgeted when no more underfunded"
    );

    category.goal_under_funded = Some((0..100000).fake());
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense
        .with_scheduled_transactions(scheduled_transactions.clone())
        .compute_amounts();

    assert_eq!(
        expense.current_amount(),
        category.budgeted + category.goal_under_funded.unwrap(),
        "Is budgeted + goal_under_funded when some underfunded"
    );

    category.goal_under_funded = None;
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense
        .with_scheduled_transactions(scheduled_transactions)
        .compute_amounts();

    // When a goal was set but no underfunded money was compouted. Maybe not even possible, but better be safe...
    assert_eq!(
        expense.current_amount(),
        0,
        "Is 0 when underfunded not defined"
    );
}

#[test]
fn compute_current_amount_when_category_has_no_goal() {
    let mut category = Category {
        goal_type: None,
        ..Faker.fake()
    };
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense.compute_amounts();

    assert_eq!(
        expense.current_amount(),
        category.budgeted,
        "Is budgeted when no scheduled transactions"
    );

    category.budgeted = (-100000..-1).fake();
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense.compute_amounts();

    assert_eq!(
        expense.current_amount(),
        0,
        "Is 0 when budgeted is negative (money moved elsewhere)"
    );

    category.budgeted = (0..100000).fake();
    let expense: Expense<Uncomputed> = category.clone().into();
    let past_transaction = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_sub_days(Days::new(1))
            .unwrap(),
        ..Faker.fake()
    };
    let expense = expense
        .with_scheduled_transactions(vec![past_transaction])
        .compute_amounts();

    assert_eq!(
        expense.current_amount(),
        category.budgeted,
        "Is budgeted when scheduled transactions are only in the past"
    );

    category.balance = (0..100).fake();
    let expense: Expense<Uncomputed> = category.clone().into();
    let past_transaction = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_sub_days(Days::new(1))
            .unwrap(),
        ..Faker.fake()
    };
    let future_transaction = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        ..Faker.fake()
    };
    let sec_future_transaction = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(10))
            .unwrap(),
        ..Faker.fake()
    };
    let expense = expense
        .with_scheduled_transactions(vec![
            past_transaction,
            future_transaction.clone(),
            sec_future_transaction.clone(),
        ])
        .compute_amounts();

    assert_eq!(
        expense.current_amount(),
        -(future_transaction.amount + sec_future_transaction.amount) - category.balance,
        "Is only total of future transactions - current balance"
    );
}
