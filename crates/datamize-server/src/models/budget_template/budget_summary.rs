use serde::{Deserialize, Serialize};

use super::{BudgetDetails, Budgeter, ComputedExpenses, ComputedSalary, TotalBudgeter};

/// A proportionally split budget's expenses.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BudgetSummary {
    budgeters: Vec<Budgeter<ComputedExpenses>>,
    total_budgeter: TotalBudgeter<ComputedExpenses>,
}

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
