use serde::{Deserialize, Serialize};
use ynab::types::ScheduledTransactionDetail;

use crate::config::PersonSalarySettings;

use super::{BudgetDetails, Budgeter, ComputedExpenses, Empty, TotalBudgeter};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommonExpenseEstimationPerPerson {
    pub name: String,
    pub salary: i64,
    pub salary_per_month: i64,
    pub proportion: f64,
    pub common_expenses: i64,
    pub individual_expenses: i64,
    pub left_over: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BudgetSummary {
    budgeters: Vec<Budgeter<ComputedExpenses>>,
    total_budgeter: TotalBudgeter<ComputedExpenses>,
}

/// A proportionally split budget's expenses.
impl BudgetSummary {
    pub fn build(
        budget_details: &BudgetDetails,
        scheduled_transactions: &[ScheduledTransactionDetail],
        person_salary_settings: Vec<PersonSalarySettings>,
    ) -> Self {
        let budgeters: Vec<_> = person_salary_settings
            .into_iter()
            .map(|pss| Budgeter::<Empty>::configure(pss).compute_salary(scheduled_transactions))
            .collect();

        let (total_budgeter, individual_expenses) = TotalBudgeter::new()
            .compute_salary(&budgeters)
            .compute_expenses(&budget_details.expenses, &budgeters);

        let budgeters: Vec<_> = budgeters
            .into_iter()
            .map(|b| b.compute_expenses(&total_budgeter, &individual_expenses))
            .collect();

        Self {
            budgeters,
            total_budgeter,
        }
    }
}
