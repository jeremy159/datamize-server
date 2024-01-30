use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    db::error::DbResult,
    models::{BudgeterConfig, ExpenseCategorization},
};

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait BudgeterConfigRepo: Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<BudgeterConfig>>;
    async fn get(&self, budgeter_id: Uuid) -> DbResult<BudgeterConfig>;
    async fn get_by_name(&self, name: &str) -> DbResult<BudgeterConfig>;
    async fn update(&self, budgeter: &BudgeterConfig) -> DbResult<()>;
    async fn delete(&self, budgeter_id: Uuid) -> DbResult<()>;
}

pub type DynBudgeterConfigRepo = Arc<dyn BudgeterConfigRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait ExpenseCategorizationRepo: Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<ExpenseCategorization>>;
    async fn get(&self, id: Uuid) -> DbResult<ExpenseCategorization>;
    async fn update_all(&self, expenses_categorization: &[ExpenseCategorization]) -> DbResult<()>;
    async fn update(&self, expense_categorization: &ExpenseCategorization) -> DbResult<()>;
}

pub type DynExpenseCategorizationRepo = Arc<dyn ExpenseCategorizationRepo>;
