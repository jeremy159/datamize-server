use serde::{Deserialize, Serialize};

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
