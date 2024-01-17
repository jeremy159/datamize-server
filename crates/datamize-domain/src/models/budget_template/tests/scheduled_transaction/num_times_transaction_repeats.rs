use chrono::{DateTime, Datelike, Local, Months, NaiveDate, TimeZone};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use ynab::RecurFrequency;

use crate::DatamizeScheduledTransaction;

#[track_caller]
fn check_method(st: &DatamizeScheduledTransaction, date: &DateTime<Local>, expected: usize) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!("check_method called from line: {}", caller_line_number);

    let count = st.get_dates_when_transaction_repeats(date).len();
    assert_eq!(count, expected);
}

#[test]
fn zero_when_no_frequency() {
    let st = DatamizeScheduledTransaction {
        frequency: None,
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 0);
}

#[test]
fn zero_when_no_frequency_that_repeats_within_a_month() {
    let now = Local::now().date_naive();
    let mut st = DatamizeScheduledTransaction {
        date_first: now.checked_sub_months(Months::new(1)).unwrap(),
        frequency: Some(RecurFrequency::EveryOtherMonth),
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 0);

    st.frequency = Some(RecurFrequency::Never);
    check_method(&st, &Local::now(), 0);

    st.frequency = Some(RecurFrequency::Every3Months);
    st.date_first = now.checked_sub_months(Months::new(1)).unwrap();
    check_method(&st, &Local::now(), 0);

    st.frequency = Some(RecurFrequency::Every4Months);
    check_method(&st, &Local::now(), 0);

    st.frequency = Some(RecurFrequency::TwiceAYear);
    check_method(&st, &Local::now(), 0);

    st.frequency = Some(RecurFrequency::Yearly);
    check_method(&st, &Local::now(), 0);

    st.frequency = Some(RecurFrequency::EveryOtherYear);
    check_method(&st, &Local::now(), 0);
}

#[test]
fn one_when_non_monthly_frequency_that_is_due_in_current_month() {
    let now = Local::now().date_naive();
    let mut st = DatamizeScheduledTransaction {
        date_first: now.checked_sub_months(Months::new(2)).unwrap(),
        frequency: Some(RecurFrequency::EveryOtherMonth),
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 1);

    st.frequency = Some(RecurFrequency::Every3Months);
    st.date_first = now.checked_sub_months(Months::new(3)).unwrap();
    check_method(&st, &Local::now(), 1);

    st.frequency = Some(RecurFrequency::Every4Months);
    st.date_first = now.checked_sub_months(Months::new(4)).unwrap();
    check_method(&st, &Local::now(), 1);

    // For now, since twice a year repeats only on june and december.
    st.frequency = Some(RecurFrequency::TwiceAYear);
    st.date_first = NaiveDate::from_ymd_opt(2023, 12, 15).unwrap();
    let jun_10 = Local::with_ymd_and_hms(&Local, 2024, 6, 10, 0, 0, 0).unwrap();
    check_method(&st, &jun_10, 1);

    st.frequency = Some(RecurFrequency::Yearly);
    st.date_first = now.checked_sub_months(Months::new(12)).unwrap();
    check_method(&st, &Local::now(), 1);

    st.frequency = Some(RecurFrequency::EveryOtherYear);
    st.date_first = now.checked_sub_months(Months::new(24)).unwrap();
    check_method(&st, &Local::now(), 1);
}

#[test]
fn is_1_when_monthly() {
    let date_first = Local::now().date_naive();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Monthly),
        date_first,
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 1);
}

#[test]
fn is_1_when_monthly_even_on_last_day() {
    let date_first = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Monthly),
        date_first,
        ..Faker.fake()
    };

    let jan_20 = Local::with_ymd_and_hms(&Local, 2024, 1, 20, 0, 0, 0).unwrap();

    check_method(&st, &jan_20, 1);
}

#[test]
fn is_2_when_twice_a_month() {
    let date_first = Local::now().date_naive().with_day(1).unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::TwiceAMonth),
        date_first,
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 2);
}

#[test]
fn is_4_when_every_week_and_starting_on_fifth_day() {
    let date_first = Local::now().date_naive().with_day(5).unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Weekly),
        date_first,
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 4);
}

#[test]
fn is_5_when_every_week_and_starting_first_day_of_month() {
    let date_first = Local::now().date_naive().with_day(1).unwrap();
    let date_first = if date_first.month0() == 1 {
        date_first.checked_sub_months(Months::new(1)).unwrap()
    } else {
        date_first
    };
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Weekly),
        date_first,
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 5);
}

#[test]
fn is_3_when_every_other_week_and_starting_beginning_of_month() {
    let date_first = Local::now().date_naive().with_day(1).unwrap();
    let date_first = if date_first.month0() == 1 {
        date_first.checked_sub_months(Months::new(1)).unwrap()
    } else {
        date_first
    };
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::EveryOtherWeek),
        date_first,
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 3);
}

#[test]
fn is_2_when_every_other_week_and_starting_fifth_day_of_month() {
    let date_first = Local::now().date_naive().with_day(5).unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::EveryOtherWeek),
        date_first,
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 2);
}

#[test]
fn is_1_when_every_4_week_and_starting_fifth_day_of_month() {
    let date_first = Local::now().date_naive().with_day(5).unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Every4Weeks),
        date_first,
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 1);
}

#[test]
fn is_2_when_every_4_week_and_starting_first_day_of_month() {
    let date_first = Local::now().date_naive().with_day(1).unwrap();
    let date_first = if date_first.month0() == 1 {
        date_first.checked_sub_months(Months::new(1)).unwrap()
    } else {
        date_first
    };
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Every4Weeks),
        date_first,
        ..Faker.fake()
    };

    check_method(&st, &Local::now(), 2);
}

#[test]
fn is_num_days_current_month_when_daily() {
    let date_first = Local::now().date_naive().with_day(1).unwrap();
    let st = DatamizeScheduledTransaction {
        frequency: Some(RecurFrequency::Daily),
        date_first,
        ..Faker.fake()
    };

    // Second day since duration difference exclude upper bound limit.
    let second_day_next_month = Local::now()
        .checked_add_months(Months::new(1))
        .and_then(|d| d.with_day(2))
        .unwrap();

    let num_days = second_day_next_month
        .signed_duration_since(Local::now().with_day(1).unwrap())
        .num_days();

    check_method(&st, &Local::now(), num_days as usize);
}
