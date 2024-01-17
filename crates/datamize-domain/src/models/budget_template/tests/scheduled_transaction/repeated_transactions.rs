use chrono::{Days, Local, Months};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use ynab::RecurFrequency;

use crate::DatamizeScheduledTransaction;

#[track_caller]
fn check_method(st: &DatamizeScheduledTransaction, expected: usize) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!("check_method called from line: {}", caller_line_number);

    let repeated = st.get_repeated_transactions();
    assert_eq!(repeated.len(), expected);
    if !repeated.is_empty() {
        for r in repeated {
            assert!(r.subtransactions.is_empty()); // Discards sub transactions
            assert_eq!(st.amount, r.amount);
            assert_eq!(st.category_name, r.category_name);
            assert_eq!(st.id, r.id);
        }
    }
}

#[test]
fn empty_when_no_frequency() {
    let st = DatamizeScheduledTransaction {
        frequency: None,
        ..Faker.fake()
    };

    check_method(&st, 0);
}

#[test]
fn empty_when_no_frequency_that_repeats_within_a_month() {
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::EveryOtherMonth),
        ..Faker.fake()
    };

    check_method(&st, 0);
}

#[test]
fn is_2_when_twice_a_month() {
    let date_first = Local::now()
        .date_naive()
        .checked_sub_days(Days::new(1))
        .unwrap();
    let date_next = Local::now()
        .date_naive()
        .checked_add_days(Days::new(7))
        .unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::TwiceAMonth),
        date_first,
        date_next,
        ..Faker.fake()
    };

    check_method(&st, 2);
}

#[test]
fn is_4_when_every_week_and_starting_yesterday() {
    let date_first = Local::now()
        .date_naive()
        .checked_sub_days(Days::new(1))
        .unwrap();
    let date_next = date_first.checked_add_days(Days::new(7)).unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Weekly),
        date_first,
        date_next,
        ..Faker.fake()
    };

    check_method(&st, 3); // Fourth one is st itself
}

#[test]
fn is_5_when_every_week_and_starting_7_days_ago() {
    let date_first = Local::now()
        .date_naive()
        .checked_sub_days(Days::new(7))
        .unwrap();
    let mut date_next = date_first.checked_add_days(Days::new(7)).unwrap();
    let current_date = Local::now();
    while date_next < current_date.date_naive() {
        date_next = date_next.checked_add_days(Days::new(7)).unwrap();
    }
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Weekly),
        date_first,
        date_next,
        ..Faker.fake()
    };

    check_method(&st, 4); // Fifth one is st itself
}

#[test]
fn is_2_when_every_other_week_and_starting_yesterday() {
    let date_first = Local::now()
        .date_naive()
        .checked_sub_days(Days::new(1))
        .unwrap();
    let date_next = date_first.checked_add_days(Days::new(14)).unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::EveryOtherWeek),
        date_first,
        date_next,
        ..Faker.fake()
    };

    check_method(&st, 1); // Second one is st itself
}

#[test]
fn is_2_when_every_other_week_and_starting_7_days_ago() {
    let date_first = Local::now()
        .date_naive()
        .checked_sub_days(Days::new(7))
        .unwrap();
    let date_next = date_first.checked_add_days(Days::new(14)).unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::EveryOtherWeek),
        date_first,
        date_next,
        ..Faker.fake()
    };

    check_method(&st, 1); // Second one is st itself
}

#[test]
fn is_1_when_every_4_week_and_next_date_in_middle() {
    let date_first = Local::now()
        .date_naive()
        .checked_sub_days(Days::new(14))
        .unwrap();
    let date_next = Local::now()
        .date_naive()
        .checked_add_days(Days::new(14))
        .unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Every4Weeks),
        date_first,
        date_next,
        ..Faker.fake()
    };

    check_method(&st, 0); // First one is st itself
}

#[test]
fn is_2_when_every_4_week_and_next_date_tomorrow() {
    let date_first = Local::now()
        .date_naive()
        .checked_sub_days(Days::new(27))
        .unwrap();
    let date_next = Local::now()
        .date_naive()
        .checked_add_days(Days::new(1))
        .unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Every4Weeks),
        date_first,
        date_next,
        ..Faker.fake()
    };

    check_method(&st, 1); // Second one is st itself
}

#[test]
fn is_num_days_current_month_when_daily() {
    let date_first = Local::now()
        .date_naive()
        .checked_sub_days(Days::new(27))
        .unwrap();
    let date_next = Local::now()
        .date_naive()
        .checked_add_days(Days::new(1))
        .unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Daily),
        date_first,
        date_next,
        ..Faker.fake()
    };

    let next_30_days = Local::now()
        .checked_add_months(Months::new(1))
        .and_then(|d| d.checked_add_days(Days::new(1)))
        .unwrap();

    let num_days = next_30_days.signed_duration_since(Local::now()).num_days();

    check_method(&st, num_days as usize - 1); // Last one is st itself
}
