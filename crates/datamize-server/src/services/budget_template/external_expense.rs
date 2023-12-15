use datamize_domain::{
    async_trait,
    db::{DbError, DynExternalExpenseRepo},
    ExternalExpense, SaveExternalExpense, Uuid,
};
use std::sync::Arc;

use crate::error::{AppError, DatamizeResult};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExternalExpenseServiceExt: Send + Sync {
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

pub type DynExternalExpenseService = Arc<dyn ExternalExpenseServiceExt>;

pub struct ExternalExpenseService {
    pub external_expense_repo: DynExternalExpenseRepo,
}

impl ExternalExpenseService {
    pub fn new_arced(external_expense_repo: DynExternalExpenseRepo) -> Arc<Self> {
        Arc::new(Self {
            external_expense_repo,
        })
    }
}

#[async_trait]
impl ExternalExpenseServiceExt for ExternalExpenseService {
    #[tracing::instrument(skip(self))]
    async fn get_all_external_expenses(&self) -> DatamizeResult<Vec<ExternalExpense>> {
        Ok(self.external_expense_repo.get_all().await?)
    }

    #[tracing::instrument(skip_all)]
    async fn create_external_expense(
        &self,
        new_expense: SaveExternalExpense,
    ) -> DatamizeResult<ExternalExpense> {
        let Err(DbError::NotFound) = self
            .external_expense_repo
            .get_by_name(&new_expense.name)
            .await
        else {
            return Err(AppError::ResourceAlreadyExist);
        };

        let external_expense: ExternalExpense = new_expense.into();
        self.external_expense_repo.update(&external_expense).await?;

        Ok(external_expense)
    }

    #[tracing::instrument(skip(self))]
    async fn get_external_expense(&self, expense_id: Uuid) -> DatamizeResult<ExternalExpense> {
        Ok(self.external_expense_repo.get(expense_id).await?)
    }

    #[tracing::instrument(skip(self, new_expense))]
    async fn update_external_expense(
        &self,
        new_expense: ExternalExpense,
    ) -> DatamizeResult<ExternalExpense> {
        let Ok(_) = self.external_expense_repo.get(new_expense.id).await else {
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
