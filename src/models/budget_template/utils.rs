use chrono::{Local, Months, NaiveDate, NaiveTime, TimeZone};
use rrule::{RRuleSet, Tz};
use ynab::types::{RecurFrequency, ScheduledTransactionDetail};

/// Check if the date is in the following 30 days, including the last day of the interval.
pub fn is_transaction_in_next_30_days(date: &NaiveDate) -> bool {
    let current_date = Local::now().date_naive();
    let next_month_date = current_date.checked_add_months(Months::new(1)).unwrap();

    date <= &next_month_date
}

/// Method to find any transactions that will be repeated more than once in a month period.
/// This means it checks from the current day of a month to next month's same day (E.g. January 15th to February 15th).
/// Those transactions will typically have a frequency of
/// * Daily
/// * Weekly
/// * EveryOtherWeek
/// * TwiceAMonth
/// * Every4Weeks
pub fn find_repeatable_transactions(
    scheduled_transaction: &ScheduledTransactionDetail,
) -> Vec<ScheduledTransactionDetail> {
    let mut repeated_transactions = vec![];

    if let Some(ref frequency) = scheduled_transaction.frequency {
        if let RecurFrequency::Daily
        | RecurFrequency::Weekly
        | RecurFrequency::EveryOtherWeek
        | RecurFrequency::TwiceAMonth
        | RecurFrequency::Every4Weeks = frequency
        {
            if let Some(rrule) = frequency.as_rfc5545_rule() {
                let date_time = Tz::Local(Local)
                    .from_local_datetime(
                        &scheduled_transaction
                            .date_next
                            .and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                    )
                    .unwrap();

                let rrule = rrule.validate(date_time).unwrap();
                let rrule_set = RRuleSet::new(date_time).rrule(rrule);
                rrule_set
                    .all(4)
                    .0
                    .into_iter()
                    .skip_while(|date| date.date_naive() == scheduled_transaction.date_next) // Skip the first iteration as it's simply the current date_next
                    .filter(|date| is_transaction_in_next_30_days(&date.date_naive()))
                    .for_each(|date| {
                        let mut new_transaction = scheduled_transaction.clone();
                        new_transaction.subtransactions = vec![];
                        new_transaction.date_next = date.date_naive();
                        repeated_transactions.push(new_transaction);
                    });
            }
        }
    }

    repeated_transactions
}
