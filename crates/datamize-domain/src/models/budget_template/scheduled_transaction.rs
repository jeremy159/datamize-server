use chrono::{DateTime, Datelike, Days, Local, Months, NaiveDate, NaiveTime, TimeZone};
use rrule::Tz;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::types::{RecurFrequency, ScheduledTransactionDetail, SubTransaction};

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatamizeScheduledTransaction {
    pub id: Uuid,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub date_first: chrono::NaiveDate,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub date_next: chrono::NaiveDate,
    pub frequency: Option<RecurFrequency>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..-1"))]
    pub amount: i64,
    pub memo: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Vec<SubTransaction>,
}

impl DatamizeScheduledTransaction {
    /// Will transform current transaction into a list of transactions.
    /// This list is composed of the transaction's children if not emtpy or else a single list of the transaction.
    pub fn flatten(self) -> Vec<Self> {
        let subtransactions: Vec<_> = self
            .subtransactions
            .iter()
            .filter(|sub_t| !sub_t.deleted)
            .cloned()
            .collect();

        match subtransactions.is_empty() {
            true => vec![self],
            false => subtransactions
                .into_iter()
                .map(|sub_t| {
                    let cloned = self.clone();
                    Self {
                        id: sub_t.id,
                        amount: sub_t.amount,
                        memo: sub_t.memo,
                        payee_id: sub_t.payee_id,
                        category_id: sub_t.category_id,
                        deleted: sub_t.deleted,
                        subtransactions: vec![],
                        category_name: None,
                        ..cloned
                    }
                })
                .collect(),
        }
    }

    pub fn with_category_name(mut self, category_name: Option<String>) -> Self {
        self.category_name = category_name;
        self
    }

    /// Check if the date is in the following 30 days, including the last day of the interval.
    pub fn is_in_next_30_days(&self) -> Option<bool> {
        let current_date = Local::now().date_naive();
        let next_month_date = current_date.checked_add_months(Months::new(1))?;

        Some(self.date_next >= current_date && self.date_next <= next_month_date)
    }

    /// Method to find any transactions that will be repeated more than once in a month period.
    /// This means it checks from the current day of a month to next month's same day (E.g. January 15th to February 15th).
    /// Those transactions will typically have a frequency of
    /// * Daily
    /// * Weekly
    /// * EveryOtherWeek
    /// * TwiceAMonth
    /// * Every4Weeks
    ///
    /// **Important** This skips the next repetition noted as `date_next` so you only get repeated transactions, not those actually coming.
    ///
    /// **Returns** an option vec because when the transaction has valid data but nothing repeats, it will return vec![],
    /// but when the data received was invalid, or there was an issue with building the rrule, it returns None,
    /// as nothing could be done to determine if future transactions repeat
    pub fn get_repeated_transactions(&self) -> Option<Vec<DatamizeScheduledTransaction>> {
        if let Some(ref frequency) = self.frequency {
            if let RecurFrequency::Daily
            | RecurFrequency::Weekly
            | RecurFrequency::EveryOtherWeek
            | RecurFrequency::TwiceAMonth
            | RecurFrequency::Every4Weeks = frequency
            {
                let date_time = NaiveTime::from_hms_opt(0, 0, 0).and_then(|time| {
                    Tz::Local(Local)
                        .from_local_datetime(&self.date_next.and_time(time))
                        .single()
                })?;

                let next_30_days =
                    Local::now()
                        .checked_add_months(Months::new(1))
                        .and_then(|d| {
                            Tz::Local(Local)
                                .from_local_datetime(&d.naive_local())
                                .single()
                        })?;

                if date_time <= next_30_days {
                    if let Some(rrule) = frequency.as_rfc5545_rule() {
                        // Range is first day included to last day included
                        let rrule = rrule.until(next_30_days);

                        let rrule_set = rrule.build(date_time).ok()?;
                        return Some(
                            rrule_set
                                .into_iter()
                                .skip_while(|date| date.date_naive() == self.date_next) // Skip the first iteration as it's simply the current date_next
                                .map(|date| DatamizeScheduledTransaction {
                                    subtransactions: vec![],
                                    date_next: date.date_naive(),
                                    ..self.clone()
                                })
                                .collect(),
                        );
                    }
                }
            }
        }

        Some(vec![])
    }

    /// Method to find any transactions that will be repeated more than once in a month might it be from previous or future days
    /// and return the dates when it happens.
    ///
    /// **Returns** an option vec because when the transaction has valid data but nothing repeats, it will return vec![],
    /// but when the data received was invalid, or there was an issue with building the rrule, it returns None,
    /// as nothing could be done to determine if future transactions repeat
    pub fn get_dates_when_transaction_repeats(
        &self,
        date: &DateTime<Local>,
    ) -> Option<Vec<NaiveDate>> {
        if let Some(ref frequency) = self.frequency {
            let last_day_curr_month = date
                .checked_add_months(Months::new(1))
                .and_then(|d| d.with_day(1))
                .and_then(|d| d.checked_sub_days(Days::new(1)))
                .and_then(|d| {
                    Tz::Local(Local)
                        .from_local_datetime(&d.naive_local())
                        .single()
                })?;

            let last_day_prev_month = date
                .naive_local()
                .with_day(1)
                .and_then(|d| d.checked_sub_days(Days::new(1)))
                .and_then(|d| Tz::Local(Local).from_local_datetime(&d).single())?;

            if self.date_first <= last_day_curr_month.date_naive() {
                if let Some(rrule) = frequency.as_rfc5545_rule() {
                    let first_date_time = NaiveTime::from_hms_opt(0, 0, 0).and_then(|time| {
                        Tz::Local(Local)
                            .from_local_datetime(&self.date_first.and_time(time))
                            .single()
                    })?;

                    let mut rrule = rrule.until(last_day_curr_month);

                    if self.date_first.day() == 31 {
                        rrule = rrule.by_month_day(vec![-1]);
                    }

                    // Range is first day included to last day included
                    let rrule_set = rrule.build(first_date_time).ok()?;

                    return Some(
                        rrule_set
                            .into_iter()
                            .filter(|d| d > &last_day_prev_month)
                            .map(|d| d.date_naive())
                            .collect(),
                    );
                }
            }
        }

        Some(vec![])
    }

    /// Method to find any transactions that was scheduled in current month, might it be from previous
    /// or future days.
    ///
    /// **Returns** an option vec because when the transaction has valid data but nothing repeats, it will return vec![],
    /// but when the data received was invalid, or there was an issue with building the rrule, it returns None,
    /// as nothing could be done to determine if future transactions repeat
    pub fn get_transactions_within_month(
        &self,
        date: &DateTime<Local>,
    ) -> Option<Vec<DatamizeScheduledTransaction>> {
        Some(
            self.get_dates_when_transaction_repeats(date)?
                .into_iter()
                .map(|d| DatamizeScheduledTransaction {
                    date_next: d,
                    ..self.clone()
                })
                .collect(),
        )
    }
}

impl From<ScheduledTransactionDetail> for DatamizeScheduledTransaction {
    fn from(value: ScheduledTransactionDetail) -> Self {
        Self {
            id: value.id,
            date_first: value.date_first,
            date_next: value.date_next,
            frequency: value.frequency,
            amount: value.amount,
            memo: value.memo,
            account_id: value.account_id,
            payee_id: value.payee_id,
            category_id: value.category_id,
            deleted: value.deleted,
            account_name: value.account_name,
            payee_name: value.payee_name,
            category_name: value.category_name,
            subtransactions: value.subtransactions,
        }
    }
}
