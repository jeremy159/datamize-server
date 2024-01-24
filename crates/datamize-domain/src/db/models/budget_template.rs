use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use uuid::Uuid;

use crate::{
    db::error::DbResult,
    models::{BudgeterConfig, ExpenseCategorization},
};

#[async_trait]
pub trait BudgeterConfigRepo: DynClone + Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<BudgeterConfig>>;
    async fn get(&self, budgeter_id: Uuid) -> DbResult<BudgeterConfig>;
    async fn get_by_name(&self, name: &str) -> DbResult<BudgeterConfig>;
    async fn update(&self, budgeter: &BudgeterConfig) -> DbResult<()>;
    async fn delete(&self, budgeter_id: Uuid) -> DbResult<()>;
}

clone_trait_object!(BudgeterConfigRepo);

pub type DynBudgeterConfigRepo = Box<dyn BudgeterConfigRepo>;

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub BudgeterConfigRepoImpl {}

    impl Clone for BudgeterConfigRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl BudgeterConfigRepo for BudgeterConfigRepoImpl {
        async fn get_all(&self) -> DbResult<Vec<BudgeterConfig>>;
        async fn get(&self, budgeter_id: Uuid) -> DbResult<BudgeterConfig>;
        async fn get_by_name(&self, name: &str) -> DbResult<BudgeterConfig>;
        async fn update(&self, budgeter: &BudgeterConfig) -> DbResult<()>;
        async fn delete(&self, budgeter_id: Uuid) -> DbResult<()>;
    }
}

#[async_trait]
pub trait ExpenseCategorizationRepo: DynClone + Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<ExpenseCategorization>>;
    async fn get(&self, id: Uuid) -> DbResult<ExpenseCategorization>;
    async fn update_all(&self, expenses_categorization: &[ExpenseCategorization]) -> DbResult<()>;
    async fn update(&self, expense_categorization: &ExpenseCategorization) -> DbResult<()>;
}

clone_trait_object!(ExpenseCategorizationRepo);

pub type DynExpenseCategorizationRepo = Box<dyn ExpenseCategorizationRepo>;

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub ExpenseCategorizationRepoImpl {}

    impl Clone for ExpenseCategorizationRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl ExpenseCategorizationRepo for ExpenseCategorizationRepoImpl {
        async fn get_all(&self) -> DbResult<Vec<ExpenseCategorization>>;
        async fn get(&self, id: Uuid) -> DbResult<ExpenseCategorization>;
        async fn update_all(
            &self,
            expenses_categorization: &[ExpenseCategorization],
        ) -> DbResult<()>;
        async fn update(&self, expense_categorization: &ExpenseCategorization) -> DbResult<()>;
    }
}
