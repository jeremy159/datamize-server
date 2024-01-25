use std::collections::HashMap;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::budget_template::expense, BudgeterConfig, BudgeterExt, BudgeterState, ComputedExpenses,
    ComputedSalary, Configured, DatamizeScheduledTransaction, Expense, SalaryFragment,
    TotalBudgeter,
};

/// A Budgeter represents someone that has income and expenses for the month.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Budgeter<S: BudgeterState> {
    #[serde(flatten)]
    extra: S,
}

impl From<BudgeterConfig> for Budgeter<Configured> {
    fn from(value: BudgeterConfig) -> Self {
        Budgeter {
            extra: Configured {
                id: value.id,
                name: value.name,
                payee_ids: value.payee_ids,
            },
        }
    }
}

impl Budgeter<Configured> {
    pub fn compute_salary(
        self,
        scheduled_transactions: &[DatamizeScheduledTransaction],
        date: &DateTime<Local>,
    ) -> Budgeter<ComputedSalary> {
        let mut fragmented_salary = HashMap::new();
        // TODO: Handle if transaction has sub transactions, e.g. DTI which included rrsp and health insurance...
        scheduled_transactions
            .iter()
            .filter(|st| match &st.payee_id {
                Some(ref id) => self.extra.payee_ids.contains(id),
                None => false,
            })
            .for_each(|st| {
                let repeats = st
                    .get_dates_when_transaction_repeats(date)
                    .unwrap_or_default()
                    .len();
                let salary_fragment = SalaryFragment {
                    payee_name: st.payee_name.clone(),
                    payee_amount: st.amount,
                    repeats: if repeats > 0 { repeats } else { 1 },
                };
                let entry = fragmented_salary
                    .entry(st.payee_id.unwrap()) // We know here payee_id is defined
                    .or_insert_with(|| Vec::with_capacity(1));
                entry.push(salary_fragment);
            });

        let salary = fragmented_salary
            .values()
            .flatten()
            .map(|fs| fs.payee_amount)
            .sum();
        let salary_month = fragmented_salary
            .values()
            .flatten()
            .map(|fs| fs.payee_amount * fs.repeats as i64)
            .sum();

        Budgeter {
            extra: ComputedSalary {
                salary,
                salary_month,
                configured: self.extra,
                fragmented_salary,
            },
        }
    }
}

impl BudgeterExt for Budgeter<Configured> {
    fn id(&self) -> Uuid {
        self.extra.id
    }

    fn name(&self) -> &str {
        &self.extra.name
    }

    fn payee_ids(&self) -> &[Uuid] {
        &self.extra.payee_ids
    }
}

impl Budgeter<ComputedSalary> {
    pub fn compute_expenses(
        self,
        total_budgeter: &TotalBudgeter<ComputedExpenses>,
        expenses: &[&Expense<expense::Computed>],
    ) -> Budgeter<ComputedExpenses> {
        let proportion = if total_budgeter.salary_month() == 0 {
            0.0
        } else {
            self.extra.salary_month as f64 / total_budgeter.salary_month() as f64
        };
        let common_expenses = (proportion * total_budgeter.common_expenses() as f64) as i64;
        let individual_expenses = expenses
            .iter()
            .filter(|e| e.name().contains(&self.extra.configured.name))
            .map(|e| e.projected_amount())
            .sum();
        let left_over = self.extra.salary_month - common_expenses - individual_expenses;

        Budgeter {
            extra: ComputedExpenses {
                common_expenses,
                proportion,
                individual_expenses,
                left_over,
                compuded_salary: self.extra,
            },
        }
    }
}

impl BudgeterExt for Budgeter<ComputedSalary> {
    fn id(&self) -> Uuid {
        self.extra.configured.id
    }

    fn name(&self) -> &str {
        &self.extra.configured.name
    }

    fn payee_ids(&self) -> &[Uuid] {
        &self.extra.configured.payee_ids
    }

    fn salary(&self) -> i64 {
        self.extra.salary
    }

    fn salary_month(&self) -> i64 {
        self.extra.salary_month
    }
}

impl BudgeterExt for Budgeter<ComputedExpenses> {
    fn id(&self) -> Uuid {
        self.extra.compuded_salary.configured.id
    }

    fn name(&self) -> &str {
        &self.extra.compuded_salary.configured.name
    }

    fn payee_ids(&self) -> &[Uuid] {
        &self.extra.compuded_salary.configured.payee_ids
    }

    fn salary(&self) -> i64 {
        self.extra.compuded_salary.salary
    }

    fn salary_month(&self) -> i64 {
        self.extra.compuded_salary.salary_month
    }

    fn proportion(&self) -> f64 {
        self.extra.proportion
    }

    fn common_expenses(&self) -> i64 {
        self.extra.common_expenses
    }

    fn individual_expenses(&self) -> i64 {
        self.extra.individual_expenses
    }

    fn left_over(&self) -> i64 {
        self.extra.left_over
    }
}

#[cfg(any(feature = "testutils", test))]
impl<S: BudgeterState + fake::Dummy<fake::Faker>> fake::Dummy<fake::Faker> for Budgeter<S> {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &fake::Faker, rng: &mut R) -> Self {
        use fake::Fake;
        let extra = config.fake_with_rng(rng);

        Self { extra }
    }
}
