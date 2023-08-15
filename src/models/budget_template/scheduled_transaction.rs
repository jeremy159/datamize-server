use chrono::{DateTime, Datelike, Local, Months, NaiveDate, NaiveTime, TimeZone};
use rrule::{RRuleSet, Tz};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::types::{RecurFrequency, ScheduledTransactionDetail, SubTransaction};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatamizeScheduledTransaction {
    pub id: Uuid,
    pub date_first: chrono::NaiveDate,
    pub date_next: chrono::NaiveDate,
    pub frequency: Option<RecurFrequency>,
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
            .cloned()
            .filter(|sub_t| !sub_t.deleted)
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
    pub fn is_in_next_30_days(&self) -> bool {
        date_is_in_next_30_days(&self.date_next)
    }

    /// Method to find any transactions that will be repeated more than once in a month period.
    /// This means it checks from the current day of a month to next month's same day (E.g. January 15th to February 15th).
    /// Those transactions will typically have a frequency of
    /// * Daily
    /// * Weekly
    /// * EveryOtherWeek
    /// * TwiceAMonth
    /// * Every4Weeks
    pub fn get_repeated_transactions(&self) -> Vec<DatamizeScheduledTransaction> {
        let mut repeated_transactions = vec![];

        if let Some(ref frequency) = self.frequency {
            if let RecurFrequency::Daily
            | RecurFrequency::Weekly
            | RecurFrequency::EveryOtherWeek
            | RecurFrequency::TwiceAMonth
            | RecurFrequency::Every4Weeks = frequency
            {
                if let Some(rrule) = frequency.as_rfc5545_rule() {
                    let date_time = Tz::Local(Local)
                        .from_local_datetime(
                            &self
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
                        .skip_while(|date| date.date_naive() == self.date_next) // Skip the first iteration as it's simply the current date_next
                        .filter(|date| date_is_in_next_30_days(&date.date_naive()))
                        .for_each(|date| {
                            let mut new_transaction = self.clone();
                            new_transaction.subtransactions = vec![];
                            new_transaction.date_next = date.date_naive();
                            repeated_transactions.push(new_transaction);
                        });
                }
            }
        }

        repeated_transactions
    }

    /// Method to find any transactions that was scheduled in current month, might it be from previous or future days.
    pub fn get_transactions_within_month(
        &self,
        date: &DateTime<Local>,
    ) -> Vec<DatamizeScheduledTransaction> {
        let mut scheduled_transactions = vec![];

        if let Some(ref frequency) = self.frequency {
            let first_day_next_month = date.checked_add_months(Months::new(1)).unwrap();

            if self.date_first < first_day_next_month.date_naive() {
                if let Some(rrule) = frequency.as_rfc5545_rule() {
                    let first_day_date_time = Tz::Local(Local)
                        .from_local_datetime(&date.naive_local())
                        .unwrap();

                    let first_date_time = Tz::Local(Local)
                        .from_local_datetime(
                            &self
                                .date_first
                                .and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                        )
                        .unwrap();

                    let first_day_next_month_date_time = Tz::Local(Local)
                        .from_local_datetime(&first_day_next_month.naive_local())
                        .unwrap();

                    let mut rrule = rrule.until(first_day_next_month_date_time);

                    if self.date_first.day() == 31 {
                        rrule = rrule.by_month_day(vec![-1]);
                    }

                    // Range is first day included but not last day
                    let rrule_set = rrule
                        .build(first_date_time)
                        .unwrap()
                        .after(first_day_date_time);

                    rrule_set.all_unchecked().into_iter().for_each(|date| {
                        let mut new_transaction = self.clone();
                        new_transaction.date_next = date.date_naive();
                        scheduled_transactions.push(new_transaction);
                    });
                }
            }
        }

        scheduled_transactions
    }
}

pub fn date_is_in_next_30_days(date: &NaiveDate) -> bool {
    let current_date = Local::now().date_naive();
    let next_month_date = current_date.checked_add_months(Months::new(1)).unwrap();

    date <= &next_month_date
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
