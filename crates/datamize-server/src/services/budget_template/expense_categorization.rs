use datamize_domain::{async_trait, db::DynExpenseCategorizationRepo, ExpenseCategorization, Uuid};
use std::sync::Arc;

use crate::error::{AppError, DatamizeResult};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExpenseCategorizationServiceExt: Send + Sync {
    async fn get_all_expenses_categorization(&self) -> DatamizeResult<Vec<ExpenseCategorization>>;
    async fn update_all_expenses_categorization(
        &self,
        new_expenses_categorization: Vec<ExpenseCategorization>,
    ) -> DatamizeResult<Vec<ExpenseCategorization>>;
    async fn get_expense_categorization(
        &self,
        expense_categorization_id: Uuid,
    ) -> DatamizeResult<ExpenseCategorization>;
    async fn update_expense_categorization(
        &self,
        new_expense_categorization: ExpenseCategorization,
    ) -> DatamizeResult<ExpenseCategorization>;
}

pub type DynExpenseCategorizationService = Arc<dyn ExpenseCategorizationServiceExt>;

pub struct ExpenseCategorizationService {
    pub expense_categorization_repo: DynExpenseCategorizationRepo,
}

impl ExpenseCategorizationService {
    pub fn new_arced(expense_categorization_repo: DynExpenseCategorizationRepo) -> Arc<Self> {
        Arc::new(Self {
            expense_categorization_repo,
        })
    }
}

#[async_trait]
impl ExpenseCategorizationServiceExt for ExpenseCategorizationService {
    #[tracing::instrument(skip(self))]
    async fn get_all_expenses_categorization(&self) -> DatamizeResult<Vec<ExpenseCategorization>> {
        Ok(self.expense_categorization_repo.get_all().await?)
    }

    #[tracing::instrument(skip_all)]
    async fn update_all_expenses_categorization(
        &self,
        new_expenses_categorization: Vec<ExpenseCategorization>,
    ) -> DatamizeResult<Vec<ExpenseCategorization>> {
        self.expense_categorization_repo
            .update_all(&new_expenses_categorization)
            .await?;

        Ok(new_expenses_categorization)
    }

    #[tracing::instrument(skip(self))]
    async fn get_expense_categorization(
        &self,
        expense_categorization_id: Uuid,
    ) -> DatamizeResult<ExpenseCategorization> {
        Ok(self
            .expense_categorization_repo
            .get(expense_categorization_id)
            .await?)
    }

    #[tracing::instrument(skip_all)]
    async fn update_expense_categorization(
        &self,
        new_expense_categorization: ExpenseCategorization,
    ) -> DatamizeResult<ExpenseCategorization> {
        let Ok(_) = self
            .expense_categorization_repo
            .get(new_expense_categorization.id)
            .await
        else {
            return Err(AppError::ResourceNotFound);
        };

        self.expense_categorization_repo
            .update(&new_expense_categorization)
            .await?;

        Ok(new_expense_categorization)
    }
}
