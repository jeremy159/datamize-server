use chrono::{Datelike, Months};

pub fn prev_month() -> String {
    chrono::Local::now()
        .with_day(1)
        .and_then(|d| d.checked_sub_months(Months::new(1)))
        .unwrap()
        .format("%B")
        .to_string()
}

pub fn curr_month() -> String {
    chrono::Local::now()
        .with_day(1)
        .unwrap()
        .format("%B")
        .to_string()
}

pub fn next_month() -> String {
    chrono::Local::now()
        .with_day(1)
        .and_then(|d| d.checked_add_months(Months::new(1)))
        .unwrap()
        .format("%B")
        .to_string()
}
