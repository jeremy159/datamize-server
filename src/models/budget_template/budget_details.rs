use std::collections::HashMap;

use chrono::{DateTime, Datelike, Local, Months, NaiveTime, TimeZone};
use rrule::Tz;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::types::{Category, ScheduledTransactionDetail};

use crate::config::BugdetCalculationDataSettings;

use super::{
    expense::Computed, Budgeter, BudgeterConfig, Configured, Expense, ExpenseType, ExternalExpense,
    PartiallyComputed, Uncomputed,
};

#[derive(Debug, Deserialize, Default, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MonthTarget {
    Previous,
    #[default]
    Current,
    Next,
}

#[derive(Debug, Deserialize, Default)]
pub struct MonthQueryParam {
    pub month: MonthTarget,
}

impl From<MonthTarget> for DateTime<Local> {
    fn from(value: MonthTarget) -> Self {
        match value {
            MonthTarget::Previous => Local::now().checked_sub_months(Months::new(1)).unwrap(),
            MonthTarget::Current => Local::now(),
            MonthTarget::Next => Local::now().checked_add_months(Months::new(1)).unwrap(),
        }
        .with_day(1)
        .unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetadata {
    /// Salary related incomes
    pub monthly_income: i64,
    /// Total income, before substracting health insurance and work-related retirement savings
    pub total_monthly_income: i64,
    /// The tartet each expense type should follow. For example, all fixed expenses shouldn't go over 60% of total income.
    pub proportion_target_per_expense_type: HashMap<ExpenseType, f64>,
}

impl Default for GlobalMetadata {
    fn default() -> Self {
        let tuples = [
            (ExpenseType::Fixed, 0.6_f64),
            (ExpenseType::Variable, 0.1_f64),
            (ExpenseType::ShortTermSaving, 0.1_f64),
            (ExpenseType::LongTermSaving, 0.1_f64),
            (ExpenseType::RetirementSaving, 0.1_f64),
        ];
        let proportion_target_per_expense_type = tuples.into_iter().collect();

        Self {
            monthly_income: 0,
            total_monthly_income: 0,
            proportion_target_per_expense_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BudgetDetails {
    global: GlobalMetadata,
    expenses: Vec<Expense<Computed>>,
}

impl BudgetDetails {
    pub fn global_metadata(&self) -> &GlobalMetadata {
        &self.global
    }

    pub fn expenses(&self) -> &[Expense<Computed>] {
        &self.expenses
    }

    pub fn build(
        categories: Vec<Category>,
        scheduled_transactions: Vec<ScheduledTransactionDetail>,
        date: &DateTime<Local>,
        external_expenses: Vec<ExternalExpense>,
        budget_calculation_data_settings: BugdetCalculationDataSettings,
        budgeters_config: Vec<BudgeterConfig>,
    ) -> Self {
        let budgeters: Vec<_> = budgeters_config
            .into_iter()
            .map(|bc| Budgeter::<Configured>::from(bc).compute_salary(&scheduled_transactions))
            .collect();

        let category_groups = budget_calculation_data_settings.category_groups;

        let mut scheduled_transactions_map =
            build_category_to_scheduled_transaction_map(scheduled_transactions, date);

        let expenses: Vec<Expense<PartiallyComputed>> = categories
            .into_iter()
            .filter(|c| !c.hidden && !c.deleted)
            .map(|c| {
                let cat_id = c.id;
                let mut expense: Expense<Uncomputed> = c.into();
                if let Some(scheduled_transactions) = scheduled_transactions_map.remove(&cat_id) {
                    expense = expense.with_scheduled_transactions(scheduled_transactions);
                }
                expense.compute_amounts()
            })
            .chain(external_expenses.into_iter().map(Expense::from))
            .collect::<Vec<_>>();

        let mut expenses: Vec<_> = expenses
            .into_iter()
            .map(|e| {
                e.set_categorization(&category_groups)
                    .set_individual_association(&budgeters)
            })
            .filter(|e| e.expense_type() != &ExpenseType::Undefined)
            .collect();

        expenses.sort_by(|first, second| first.sub_expense_type().cmp(second.sub_expense_type()));

        let monthly_income = budgeters.iter().map(|b| b.salary_month()).sum();
        let health_insurance_amount = expenses
            .iter()
            .filter(|e| e.name().contains("Assurance Santé"))
            .map(|e| e.projected_amount())
            .sum::<i64>();
        let retirement_savings_total = expenses
            .iter()
            .filter(|e| e.expense_type() == &ExpenseType::RetirementSaving)
            .map(|e| e.projected_amount())
            .sum::<i64>();

        let total_monthly_income =
            monthly_income + health_insurance_amount + retirement_savings_total;

        let expenses = expenses
            .into_iter()
            .map(|e| e.compute_proportions(total_monthly_income))
            .collect();

        Self {
            global: GlobalMetadata {
                monthly_income,
                total_monthly_income,
                ..Default::default()
            },
            expenses,
        }
    }
}

fn build_category_to_scheduled_transaction_map(
    scheduled_transactions: Vec<ScheduledTransactionDetail>,
    date: &DateTime<Local>,
) -> HashMap<Uuid, Vec<ScheduledTransactionDetail>> {
    let mut hash_map: HashMap<Uuid, Vec<ScheduledTransactionDetail>> =
        HashMap::with_capacity(scheduled_transactions.len());

    let scheduled_transactions_filtered: Vec<ScheduledTransactionDetail> = scheduled_transactions
        .into_iter()
        .filter(|st| st.category_id.is_some())
        .filter(|st| !st.deleted)
        .flat_map(|st| get_scheduled_transactions_within_month(&st, date))
        .collect();

    for st in flatten_sub_transactions(scheduled_transactions_filtered) {
        if let Some(category_id) = st.category_id {
            hash_map
                .entry(category_id)
                .or_insert_with(Vec::new)
                .push(st);
        }
    }

    hash_map
}

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
