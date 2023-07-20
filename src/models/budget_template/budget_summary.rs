use serde::{Deserialize, Serialize};

use super::{BudgetDetails, Budgeter, ComputedExpenses, ComputedSalary, TotalBudgeter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSummary {
    budgeters: Vec<Budgeter<ComputedExpenses>>,
    total_budgeter: TotalBudgeter<ComputedExpenses>,
}

/// A proportionally split budget's expenses.
impl BudgetSummary {
    pub fn build(budget_details: &BudgetDetails, budgeters: Vec<Budgeter<ComputedSalary>>) -> Self {
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
