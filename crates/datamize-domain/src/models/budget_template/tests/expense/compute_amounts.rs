use chrono::{Datelike, Days, Local, Months};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
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

#[derive(Debug, Clone)]
struct ExpectedProjected {
    projected_amount: i64,
}

#[track_caller]
fn check_method_projected_amount(
    expense: impl Into<Expense<Uncomputed>>,
    st: Vec<DatamizeScheduledTransaction>,
    ExpectedProjected { projected_amount }: ExpectedProjected,
    panic_msg: Option<&str>,
) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!(
        "check_method_projected_amount called from line: {}",
        caller_line_number
    );

    let expense: Expense<Uncomputed> = expense.into();
    let expense = expense
        .with_scheduled_transactions(st)
        .build_dates()
        .compute_amounts();

    match panic_msg {
        None => assert_eq!(expense.projected_amount(), projected_amount),
        Some(msg) => assert_eq!(expense.projected_amount(), projected_amount, "{}", msg),
    };
}

#[derive(Debug, Clone)]
struct ExpectedCurrent {
    current_amount: i64,
}

#[track_caller]
fn check_method_current_amount(
    expense: impl Into<Expense<Uncomputed>>,
    st: Vec<DatamizeScheduledTransaction>,
    ExpectedCurrent { current_amount }: ExpectedCurrent,
    panic_msg: Option<&str>,
) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!(
        "check_method_current_amount called from line: {}",
        caller_line_number
    );

    let expense: Expense<Uncomputed> = expense.into();
    let expense = expense.with_scheduled_transactions(st).compute_amounts();

    match panic_msg {
        None => assert_eq!(expense.current_amount(), current_amount),
        Some(msg) => assert_eq!(expense.current_amount(), current_amount, "{}", msg),
    };
}

#[test]
fn compute_projected_amount_is_goal_target_when_goal_not_plan_spending() {
    let goals = [
        GoalType::Debt,
        GoalType::MonthlyFunding,
        GoalType::TargetBalance,
        GoalType::TargetBalanceByDate,
    ];
    let category = Category {
        goal_type: Some(goals.choose(&mut rand::thread_rng()).unwrap().to_owned()),
        goal_target: Some((0..100000).fake()),
        ..Faker.fake()
    };
    let st = fake::vec![DatamizeScheduledTransaction; 1];
    let goal_target = category.goal_target.unwrap();

    check_method_projected_amount(
        category,
        st.clone(),
        ExpectedProjected {
            projected_amount: goal_target,
        },
        None,
    );
}

#[test]
fn compute_projected_amount_when_goal_target_is_plan_spending_and_cadence_monthly() {
    let mut category = Category {
        goal_type: Some(GoalType::PlanYourSpending),
        goal_cadence: Some(1),
        goal_cadence_frequency: Some((1..13).fake()),
        goal_target: Some((0..100000).fake()),
        ..Faker.fake()
    };
    let goal_target = category.goal_target.unwrap();
    let goal_cadence_frequency = category.goal_cadence_frequency.unwrap() as i64;
    check_method_projected_amount(
        category.clone(),
        vec![],
        ExpectedProjected {
            projected_amount: goal_target / goal_cadence_frequency,
        },
        Some("Is goal target divided by frequency"),
    );

    category.goal_cadence_frequency = None;
    let goal_target = category.goal_target.unwrap();
    check_method_projected_amount(
        category.clone(),
        vec![],
        ExpectedProjected {
            projected_amount: goal_target,
        },
        Some("Is goal target when no frequency is set"),
    );

    category.goal_cadence_frequency = Some(0);
    check_method_projected_amount(
        category.clone(),
        vec![],
        ExpectedProjected {
            projected_amount: 0,
        },
        Some("Is 0 when frequency is wrongly set"),
    );
}

#[test]
fn compute_projected_amount_when_goal_target_is_plan_spending_and_cadence_weekly() {
    let date_first = Local::now().date_naive().with_day(1).unwrap();
    let date_first = if date_first.month0() == 1 {
        date_first.checked_sub_months(Months::new(1)).unwrap()
    } else {
        date_first
    };
    let mut category = Category {
        goal_type: Some(GoalType::PlanYourSpending),
        goal_cadence: Some(2),
        goal_cadence_frequency: Some(1),
        goal_day: Some(date_first.weekday().num_days_from_sunday() as i32),
        goal_creation_month: Some(date_first),
        goal_target: Some((0..100000).fake()),
        ..Faker.fake()
    };
    let goal_target = category.goal_target.unwrap();
    check_method_projected_amount(
        category.clone(),
        vec![],
        ExpectedProjected {
            projected_amount: goal_target * 5,
        },
        Some("Is goal target times 5 when goal repeats weekly starting first day of month"),
    );

    category.goal_creation_month = Some(date_first.checked_add_days(Days::new(7)).unwrap());
    let goal_target = category.goal_target.unwrap();
    check_method_projected_amount(
        category.clone(),
        vec![],
        ExpectedProjected {
            projected_amount: goal_target * 4,
        },
        Some("Is goal target times 4 when goal repeats weekly starting first 7 days into month"),
    );

    category.goal_creation_month = Some(date_first);
    category.goal_cadence_frequency = Some(2);
    let goal_target = category.goal_target.unwrap();
    check_method_projected_amount(
        category.clone(),
        vec![],
        ExpectedProjected {
            projected_amount: goal_target * 3,
        },
        Some(
            "Is goal target times 3 when goal repeats every other week starting first day of month",
        ),
    );

    category.goal_cadence_frequency = None;
    check_method_projected_amount(
        category.clone(),
        vec![],
        ExpectedProjected {
            projected_amount: 0,
        },
        Some("Is 0 when no frequency is set"),
    );

    category.goal_cadence_frequency = Some(0);
    check_method_projected_amount(
        category.clone(),
        vec![],
        ExpectedProjected {
            projected_amount: 0,
        },
        Some("Is 0 when frequency is wrongly set"),
    );
}

#[test]
fn compute_projected_amount_when_goal_target_is_plan_spending_and_cadence_every_2_years() {
    let category = Category {
        goal_type: Some(GoalType::PlanYourSpending),
        goal_cadence: Some(14),
        goal_cadence_frequency: Faker.fake(),
        goal_target: Some((0..100000).fake()),
        ..Faker.fake()
    };
    let goal_target = category.goal_target.unwrap();
    check_method_projected_amount(
        category,
        vec![],
        ExpectedProjected {
            projected_amount: goal_target / 24,
        },
        Some("Is goal target divided by 24 months"),
    );
}

#[test]
fn compute_projected_amount_when_goal_target_is_plan_spending_and_cadence_every_x_months() {
    let cadence = (3..=13).fake();

    let category = Category {
        goal_type: Some(GoalType::PlanYourSpending),
        goal_cadence: Some(cadence),
        goal_cadence_frequency: Faker.fake(),
        goal_target: Some((0..100000).fake()),
        ..Faker.fake()
    };
    let goal_target = category.goal_target.unwrap();
    check_method_projected_amount(
        category,
        vec![],
        ExpectedProjected {
            projected_amount: goal_target / (cadence - 1) as i64,
        },
        Some("Is goal target divided by cadence minus 1"),
    );
}

#[test]
fn compute_current_amount_is_inverted_total_of_scheduled_transactions() {
    let category = Category {
        goal_type: None,
        ..Faker.fake()
    };
    let mut st = fake::vec![DatamizeScheduledTransaction; 1];
    st[0].amount = -category.budgeted;
    check_method_current_amount(
        category,
        st.clone(),
        ExpectedCurrent {
            current_amount: st.into_iter().map(|st| -st.amount).sum(),
        },
        None,
    );
}

#[test]
fn compute_current_amount_when_category_has_goal() {
    let mut category = Category {
        goal_type: Some(Faker.fake()),
        goal_under_funded: Some(0),
        ..Faker.fake()
    };
    let st = fake::vec![DatamizeScheduledTransaction; 1..3];
    let budgeted = category.budgeted;
    check_method_current_amount(
        category.clone(),
        st.clone(),
        ExpectedCurrent {
            current_amount: budgeted,
        },
        Some("Is budgeted when no more underfunded"),
    );

    category.goal_under_funded = Some((0..100000).fake());
    let budgeted = category.budgeted;
    let goal_under_funded = category.goal_under_funded.unwrap();
    check_method_current_amount(
        category.clone(),
        st.clone(),
        ExpectedCurrent {
            current_amount: budgeted + goal_under_funded,
        },
        Some("Is budgeted + goal_under_funded when some underfunded"),
    );

    category.goal_under_funded = None;
    check_method_current_amount(
        category,
        st,
        ExpectedCurrent { current_amount: 0 },
        Some("Is 0 when underfunded not defined"),
    );
}

#[test]
fn compute_current_amount_when_category_has_no_goal() {
    let mut category = Category {
        goal_type: None,
        ..Faker.fake()
    };
    let budgeted = category.budgeted;
    check_method_current_amount(
        category.clone(),
        vec![],
        ExpectedCurrent {
            current_amount: budgeted,
        },
        Some("Is budgeted when no scheduled transactions"),
    );

    category.budgeted = (-100000..-1).fake();
    check_method_current_amount(
        category.clone(),
        vec![],
        ExpectedCurrent { current_amount: 0 },
        Some("Is 0 when budgeted is negative (money moved elsewhere)"),
    );

    category.budgeted = (0..100000).fake();
    let past_transaction = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_sub_days(Days::new(1))
            .unwrap(),
        ..Faker.fake()
    };
    let budgeted = category.budgeted;
    check_method_current_amount(
        category.clone(),
        vec![past_transaction],
        ExpectedCurrent {
            current_amount: budgeted,
        },
        Some("Is budgeted when scheduled transactions are only in the past"),
    );

    category.balance = (0..100).fake();
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
    let balance = category.balance;
    let first_trans_amount = future_transaction.amount;
    let sec_trans_amount = sec_future_transaction.amount;
    check_method_current_amount(
        category,
        vec![past_transaction, future_transaction, sec_future_transaction],
        ExpectedCurrent {
            current_amount: -(first_trans_amount + sec_trans_amount) - balance,
        },
        Some("Is only total of future transactions - current balance"),
    );
}
