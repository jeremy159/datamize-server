use std::collections::HashMap;

use chrono::{DateTime, Datelike, Local, Months, NaiveTime, TimeZone};
use rrule::Tz;
use uuid::Uuid;
use ynab::types::ScheduledTransactionDetail;

use crate::{
    config::PersonSalarySettings,
    models::budget_template::{find_repeatable_transactions, CommonExpenseEstimationPerPerson},
};

/// Method to find any transactions that was scheduled in current month, might it be from previous or future days.
pub fn get_scheduled_transactions_within_month(
    scheduled_transaction: &ScheduledTransactionDetail,
    date: &DateTime<Local>,
) -> Vec<ScheduledTransactionDetail> {
    let mut scheduled_transactions = vec![];

    if let Some(ref frequency) = scheduled_transaction.frequency {
        let first_day_next_month = date.checked_add_months(Months::new(1)).unwrap();

        if scheduled_transaction.date_first < first_day_next_month.date_naive() {
            if let Some(rrule) = frequency.as_rfc5545_rule() {
                let first_day_date_time = Tz::Local(Local)
                    .from_local_datetime(&date.naive_local())
                    .unwrap();

                let first_date_time = Tz::Local(Local)
                    .from_local_datetime(
                        &scheduled_transaction
                            .date_first
                            .and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                    )
                    .unwrap();

                let first_day_next_month_date_time = Tz::Local(Local)
                    .from_local_datetime(&first_day_next_month.naive_local())
                    .unwrap();

                let mut rrule = rrule.until(first_day_next_month_date_time);

                if scheduled_transaction.date_first.day() == 31 {
                    rrule = rrule.by_month_day(vec![-1]);
                }

                // Range is first day included but not last day
                let rrule_set = rrule
                    .build(first_date_time)
                    .unwrap()
                    .after(first_day_date_time);

                rrule_set.all_unchecked().into_iter().for_each(|date| {
                    let mut new_transaction = scheduled_transaction.clone();
                    new_transaction.date_next = date.date_naive();
                    scheduled_transactions.push(new_transaction);
                });
            }
        }
    }

    scheduled_transactions
}

pub fn flatten_sub_transactions(
    scheduled_transaction: Vec<ScheduledTransactionDetail>,
) -> Vec<ScheduledTransactionDetail> {
    scheduled_transaction
        .into_iter()
        .flat_map(|st| {
            match st
                .subtransactions
                .iter()
                .filter(|sub_st| !sub_st.deleted)
                .count()
            {
                0 => vec![st],
                _ => st
                    .subtransactions
                    .iter()
                    .filter(|sub_st| !sub_st.deleted)
                    .map(|sub_st| st.clone().from_subtransaction(sub_st))
                    .collect(),
            }
        })
        .collect()
}

pub fn get_salary_per_person_per_month(
    scheduled_transactions: &[ScheduledTransactionDetail],
    person_salary_settings: &[PersonSalarySettings],
) -> Vec<CommonExpenseEstimationPerPerson> {
    let mut output: HashMap<String, CommonExpenseEstimationPerPerson> = HashMap::new();
    let salary_payee_ids: Vec<&Uuid> = person_salary_settings
        .iter()
        .flat_map(|ps| &ps.payee_ids)
        .collect();

    for st in scheduled_transactions
        .iter()
        .filter(|st| salary_payee_ids.contains(&&st.payee_id.unwrap()))
    {
        let name = person_salary_settings
            .iter()
            .find(|ps| ps.payee_ids.contains(&st.payee_id.unwrap()))
            .unwrap()
            .name
            .clone();
        let ps = CommonExpenseEstimationPerPerson {
            name: name.clone(),
            salary: st.amount,
            salary_per_month: st.amount
                + find_repeatable_transactions(st)
                    .iter()
                    .map(|v| v.amount)
                    .sum::<i64>(),
            ..Default::default()
        };
        output
            .entry(name)
            .and_modify(|o| {
                o.salary_per_month += ps.salary_per_month;
            })
            .or_insert_with(|| ps);
    }

    output.into_values().collect()
}

pub fn get_subtransactions_category_ids(
    scheduled_transactions: &[ScheduledTransactionDetail],
) -> Vec<uuid::Uuid> {
    scheduled_transactions
        .iter()
        .flat_map(|st| &st.subtransactions)
        .filter_map(|sub_st| sub_st.category_id.to_owned())
        .collect()
}
