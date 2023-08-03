use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    db::budget_template::ExpenseCategorizationRepo,
    error::{AppError, DatamizeResult},
    models::budget_template::ExpenseCategorization,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExpenseCategorizationServiceExt {
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

pub struct ExpenseCategorizationService<ECR: ExpenseCategorizationRepo> {
    pub expense_categorization_repo: ECR,
}

#[async_trait]
impl<ECR> ExpenseCategorizationServiceExt for ExpenseCategorizationService<ECR>
where
    ECR: ExpenseCategorizationRepo + Sync + Send,
{
    #[tracing::instrument(skip(self))]
    async fn get_all_expenses_categorization(&self) -> DatamizeResult<Vec<ExpenseCategorization>> {
        self.expense_categorization_repo.get_all().await
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
        self.expense_categorization_repo
            .get(expense_categorization_id)
            .await
    }

    #[tracing::instrument(skip_all)]
    async fn update_expense_categorization(
        &self,
        new_expense_categorization: ExpenseCategorization,
    ) -> DatamizeResult<ExpenseCategorization> {
        let Ok(_) = self.expense_categorization_repo.get(new_expense_categorization.id).await else {
            return Err(AppError::ResourceNotFound);
        };

        self.expense_categorization_repo
            .update(&new_expense_categorization)
            .await?;

        Ok(new_expense_categorization)
    }
}
