use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    error::DatamizeResult,
    models::budget_template::{BudgeterConfig, ExpenseCategorization, ExternalExpense},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BudgeterConfigRepo {
    async fn get_all(&self) -> DatamizeResult<Vec<BudgeterConfig>>;
    async fn get(&self, budgeter_id: Uuid) -> DatamizeResult<BudgeterConfig>;
    async fn get_by_name(&self, name: &str) -> DatamizeResult<BudgeterConfig>;
    async fn update(&self, budgeter: &BudgeterConfig) -> DatamizeResult<()>;
    async fn delete(&self, budgeter_id: Uuid) -> DatamizeResult<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExternalExpenseRepo {
    async fn get_all(&self) -> DatamizeResult<Vec<ExternalExpense>>;
    async fn get(&self, expense_id: Uuid) -> DatamizeResult<ExternalExpense>;
    async fn get_by_name(&self, name: &str) -> DatamizeResult<ExternalExpense>;
    async fn update(&self, expense: &ExternalExpense) -> DatamizeResult<()>;
    async fn delete(&self, expense_id: Uuid) -> DatamizeResult<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExpenseCategorizationRepo {
    async fn get_all(&self) -> DatamizeResult<Vec<ExpenseCategorization>>;
    async fn get(&self, id: Uuid) -> DatamizeResult<ExpenseCategorization>;
    async fn update_all(
        &self,
        expenses_categorization: &[ExpenseCategorization],
    ) -> DatamizeResult<()>;
    async fn update(&self, expense_categorization: &ExpenseCategorization) -> DatamizeResult<()>;
}
