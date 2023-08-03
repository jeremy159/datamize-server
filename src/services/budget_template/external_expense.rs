use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    db::budget_template::ExternalExpenseRepo,
    error::{AppError, DatamizeResult},
    models::budget_template::{ExternalExpense, SaveExternalExpense},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExternalExpenseServiceExt {
    async fn get_all_external_expenses(&self) -> DatamizeResult<Vec<ExternalExpense>>;
    async fn create_external_expense(
        &self,
        new_expense: SaveExternalExpense,
    ) -> DatamizeResult<ExternalExpense>;
    async fn get_external_expense(&self, expense_id: Uuid) -> DatamizeResult<ExternalExpense>;
    async fn update_external_expense(
        &self,
        new_expense: ExternalExpense,
    ) -> DatamizeResult<ExternalExpense>;
    async fn delete_external_expense(&self, expense_id: Uuid) -> DatamizeResult<ExternalExpense>;
}

pub struct ExternalExpenseService<EER: ExternalExpenseRepo> {
    pub external_expense_repo: EER,
}

#[async_trait]
impl<EER> ExternalExpenseServiceExt for ExternalExpenseService<EER>
where
    EER: ExternalExpenseRepo + Sync + Send,
{
    #[tracing::instrument(skip(self))]
    async fn get_all_external_expenses(&self) -> DatamizeResult<Vec<ExternalExpense>> {
        self.external_expense_repo.get_all().await
    }

    #[tracing::instrument(skip_all)]
    async fn create_external_expense(
        &self,
        new_expense: SaveExternalExpense,
    ) -> DatamizeResult<ExternalExpense> {
        let Err(AppError::ResourceNotFound) = self.external_expense_repo.get_by_name(&new_expense.name).await else
        {
            return Err(AppError::ResourceAlreadyExist);
        };

        let external_expense: ExternalExpense = new_expense.into();
        self.external_expense_repo.update(&external_expense).await?;

        Ok(external_expense)
    }

    #[tracing::instrument(skip(self))]
    async fn get_external_expense(&self, expense_id: Uuid) -> DatamizeResult<ExternalExpense> {
        self.external_expense_repo.get(expense_id).await
    }

    #[tracing::instrument(skip(self, new_expense))]
    async fn update_external_expense(
        &self,
        new_expense: ExternalExpense,
    ) -> DatamizeResult<ExternalExpense> {
        let Ok(_) =  self.external_expense_repo.get(new_expense.id).await else {
                return Err(AppError::ResourceNotFound);
            };

        self.external_expense_repo.update(&new_expense).await?;

        Ok(new_expense)
    }

    #[tracing::instrument(skip(self))]
    async fn delete_external_expense(&self, expense_id: Uuid) -> DatamizeResult<ExternalExpense> {
        let Ok(external_expense) = self.external_expense_repo.get(expense_id).await else {
            return Err(AppError::ResourceNotFound);
        };

        self.external_expense_repo.delete(expense_id).await?;

        Ok(external_expense)
    }
}
