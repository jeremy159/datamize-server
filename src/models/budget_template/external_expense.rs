use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ExpenseType, SubExpenseType};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ExternalExpense {
    pub id: Uuid,
    pub name: String,
    /// The type the expense relates to.
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub expense_type: ExpenseType,
    /// The sub_type the expense relates to. This can be useful for example to group only housing expenses together.
    #[serde(rename = "sub_type")]
    #[sqlx(rename = "sub_type")]
    pub sub_expense_type: SubExpenseType,
    /// Will either be the goal_under_funded or the amount of the linked scheduled transaction coming in the month
    pub projected_amount: i64,
}

impl ExternalExpense {
    pub fn new(
        name: String,
        expense_type: ExpenseType,
        sub_expense_type: SubExpenseType,
        projected_amount: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            expense_type,
            sub_expense_type,
            projected_amount,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaveExternalExpense {
    pub name: String,
    /// The type the expense relates to.
    #[serde(rename = "type")]
    pub expense_type: ExpenseType,
    /// The sub_type the expense relates to. This can be useful for example to group only housing expenses together.
    #[serde(rename = "sub_type")]
    pub sub_expense_type: SubExpenseType,
    /// Will either be the goal_under_funded or the amount of the linked scheduled transaction coming in the month
    pub projected_amount: i64,
}

impl From<SaveExternalExpense> for ExternalExpense {
    fn from(value: SaveExternalExpense) -> Self {
        Self::new(
            value.name,
            value.expense_type,
            value.sub_expense_type,
            value.projected_amount,
        )
    }
}
