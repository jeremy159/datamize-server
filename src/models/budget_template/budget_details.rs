use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{Expense, ExpenseType};

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
    pub global: GlobalMetadata,
    pub expenses: Vec<Expense>,
}
