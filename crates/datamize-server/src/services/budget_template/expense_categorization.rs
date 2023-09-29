use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    db::budget_template::DynExpenseCategorizationRepo,
    error::{AppError, DatamizeResult},
    models::budget_template::ExpenseCategorization,
};

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

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;
    use crate::db::budget_template::MockExpenseCategorizationRepoImpl;

    #[tokio::test]
    async fn update_expense_categorization_success() {
        let expense_categorization = Faker.fake::<ExpenseCategorization>();
        let new_expense_categorization = expense_categorization.clone();
        let mut expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());
        expense_categorization_repo
            .expect_get()
            .once()
            .return_once(|_| Ok(new_expense_categorization));
        expense_categorization_repo
            .expect_update()
            .once()
            .returning(|_| Ok(()));

        let expense_categorization_service = ExpenseCategorizationService {
            expense_categorization_repo,
        };

        let actual = expense_categorization_service
            .update_expense_categorization(expense_categorization.clone())
            .await
            .unwrap();
        assert_eq!(expense_categorization, actual);
    }

    #[tokio::test]
    async fn update_expense_categorization_failure_does_not_exist() {
        let expense_categorization = Faker.fake::<ExpenseCategorization>();
        let mut expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());
        expense_categorization_repo
            .expect_get()
            .return_once(|_| Err(AppError::ResourceNotFound));
        expense_categorization_repo.expect_update().never();

        let expense_categorization_service = ExpenseCategorizationService {
            expense_categorization_repo,
        };

        let actual = expense_categorization_service
            .update_expense_categorization(expense_categorization.clone())
            .await;
        assert!(matches!(actual, Err(AppError::ResourceNotFound)));
    }
}
