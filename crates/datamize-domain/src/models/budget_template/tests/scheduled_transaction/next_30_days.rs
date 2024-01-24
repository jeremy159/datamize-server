use chrono::{Days, Local, Months};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;

use crate::DatamizeScheduledTransaction;

#[track_caller]
fn check_method(st: &DatamizeScheduledTransaction, expected: bool) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!("check_method called from line: {}", caller_line_number);

    assert_eq!(st.is_in_next_30_days().unwrap(), expected);
}

#[test]
fn date_is_in_past() {
    let st = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_sub_days(Days::new(1))
            .unwrap(),
        ..Faker.fake()
    };

    check_method(&st, false);
}

#[test]
fn date_is_too_far_in_future() {
    let st = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .and_then(|d| d.checked_add_months(Months::new(1)))
            .unwrap(),
        ..Faker.fake()
    };

    check_method(&st, false);
}

#[test]
fn date_is_in_next_30_days() {
    let st = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        ..Faker.fake()
    };

    check_method(&st, true);

    let st = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_months(Months::new(1))
            .and_then(|d| d.checked_sub_days(Days::new(1)))
            .unwrap(),
        ..Faker.fake()
    };

    check_method(&st, true);
}
