use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use uuid::Uuid;

use crate::{
    error::DatamizeResult,
    models::budget_template::{BudgeterConfig, ExpenseCategorization, ExternalExpense},
};

#[async_trait]
pub trait BudgeterConfigRepo: DynClone {
    async fn get_all(&self) -> DatamizeResult<Vec<BudgeterConfig>>;
    async fn get(&self, budgeter_id: Uuid) -> DatamizeResult<BudgeterConfig>;
    async fn get_by_name(&self, name: &str) -> DatamizeResult<BudgeterConfig>;
    async fn update(&self, budgeter: &BudgeterConfig) -> DatamizeResult<()>;
    async fn delete(&self, budgeter_id: Uuid) -> DatamizeResult<()>;
}

clone_trait_object!(BudgeterConfigRepo);

pub type DynBudgeterConfigRepo = Box<dyn BudgeterConfigRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub BudgeterConfigRepoImpl {}

    impl Clone for BudgeterConfigRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl BudgeterConfigRepo for BudgeterConfigRepoImpl {
        async fn get_all(&self) -> DatamizeResult<Vec<BudgeterConfig>>;
        async fn get(&self, budgeter_id: Uuid) -> DatamizeResult<BudgeterConfig>;
        async fn get_by_name(&self, name: &str) -> DatamizeResult<BudgeterConfig>;
        async fn update(&self, budgeter: &BudgeterConfig) -> DatamizeResult<()>;
        async fn delete(&self, budgeter_id: Uuid) -> DatamizeResult<()>;
    }
}

#[async_trait]
pub trait ExternalExpenseRepo: DynClone {
    async fn get_all(&self) -> DatamizeResult<Vec<ExternalExpense>>;
    async fn get(&self, expense_id: Uuid) -> DatamizeResult<ExternalExpense>;
    async fn get_by_name(&self, name: &str) -> DatamizeResult<ExternalExpense>;
    async fn update(&self, expense: &ExternalExpense) -> DatamizeResult<()>;
    async fn delete(&self, expense_id: Uuid) -> DatamizeResult<()>;
}

clone_trait_object!(ExternalExpenseRepo);

pub type DynExternalExpenseRepo = Box<dyn ExternalExpenseRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub ExternalExpenseRepoImpl {}

    impl Clone for ExternalExpenseRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl ExternalExpenseRepo for ExternalExpenseRepoImpl {
        async fn get_all(&self) -> DatamizeResult<Vec<ExternalExpense>>;
        async fn get(&self, expense_id: Uuid) -> DatamizeResult<ExternalExpense>;
        async fn get_by_name(&self, name: &str) -> DatamizeResult<ExternalExpense>;
        async fn update(&self, expense: &ExternalExpense) -> DatamizeResult<()>;
        async fn delete(&self, expense_id: Uuid) -> DatamizeResult<()>;
    }
}

#[async_trait]
pub trait ExpenseCategorizationRepo: DynClone {
    async fn get_all(&self) -> DatamizeResult<Vec<ExpenseCategorization>>;
    async fn get(&self, id: Uuid) -> DatamizeResult<ExpenseCategorization>;
    async fn update_all(
        &self,
        expenses_categorization: &[ExpenseCategorization],
    ) -> DatamizeResult<()>;
    async fn update(&self, expense_categorization: &ExpenseCategorization) -> DatamizeResult<()>;
}

clone_trait_object!(ExpenseCategorizationRepo);

pub type DynExpenseCategorizationRepo = Box<dyn ExpenseCategorizationRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub ExpenseCategorizationRepoImpl {}

    impl Clone for ExpenseCategorizationRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl ExpenseCategorizationRepo for ExpenseCategorizationRepoImpl {
        async fn get_all(&self) -> DatamizeResult<Vec<ExpenseCategorization>>;
        async fn get(&self, id: Uuid) -> DatamizeResult<ExpenseCategorization>;
        async fn update_all(
            &self,
            expenses_categorization: &[ExpenseCategorization],
        ) -> DatamizeResult<()>;
        async fn update(&self, expense_categorization: &ExpenseCategorization) -> DatamizeResult<()>;
    }
}
