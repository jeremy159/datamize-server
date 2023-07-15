use serde::{Deserialize, Serialize};
use ynab::types::ScheduledTransactionDetail;

use super::{BudgetDetails, Budgeter, BudgeterConfig, ComputedExpenses, Configured, TotalBudgeter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSummary {
    budgeters: Vec<Budgeter<ComputedExpenses>>,
    total_budgeter: TotalBudgeter<ComputedExpenses>,
}

/// A proportionally split budget's expenses.
impl BudgetSummary {
    pub fn build(
        budget_details: &BudgetDetails,
        scheduled_transactions: &[ScheduledTransactionDetail],
        budgeters_config: Vec<BudgeterConfig>,
    ) -> Self {
        let budgeters: Vec<_> = budgeters_config
            .into_iter()
            .map(|bc| Budgeter::<Configured>::from(bc).compute_salary(scheduled_transactions))
            .collect();

        let (total_budgeter, individual_expenses) = TotalBudgeter::new()
            .compute_salary(&budgeters)
            .compute_expenses(budget_details.expenses(), &budgeters);

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
