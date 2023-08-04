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

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;
    use crate::db::budget_template::MockExternalExpenseRepo;

    #[tokio::test]
    async fn create_external_expense_success() {
        let mut external_expense_repo = MockExternalExpenseRepo::new();
        external_expense_repo
            .expect_get_by_name()
            .once()
            .returning(|_| Err(AppError::ResourceNotFound));
        external_expense_repo
            .expect_update()
            .once()
            .returning(|_| Ok(()));

        let external_expense_service = ExternalExpenseService {
            external_expense_repo,
        };

        let new_external_expense: SaveExternalExpense = Faker.fake();

        let expected: ExternalExpense = new_external_expense.clone().into();
        let actual = external_expense_service
            .create_external_expense(new_external_expense)
            .await
            .unwrap();
        assert_eq!(expected.name, actual.name);
        assert_eq!(expected.expense_type, actual.expense_type);
        assert_eq!(expected.sub_expense_type, actual.sub_expense_type);
        assert_eq!(expected.projected_amount, actual.projected_amount);
    }

    #[tokio::test]
    async fn create_external_expense_failure_already_exist() {
        let mut external_expense_repo = MockExternalExpenseRepo::new();
        external_expense_repo
            .expect_get_by_name()
            .once()
            .returning(|_| Ok(Faker.fake::<ExternalExpense>()));
        external_expense_repo.expect_update().never();

        let external_expense_service = ExternalExpenseService {
            external_expense_repo,
        };

        let new_external_expense: SaveExternalExpense = Faker.fake();

        let actual = external_expense_service
            .create_external_expense(new_external_expense)
            .await;
        assert!(matches!(actual, Err(AppError::ResourceAlreadyExist)));
    }

    #[tokio::test]
    async fn update_external_expense_success() {
        let external_expense = Faker.fake::<ExternalExpense>();
        let new_external_expense = external_expense.clone();
        let mut external_expense_repo = MockExternalExpenseRepo::new();
        external_expense_repo
            .expect_get()
            .return_once(|_| Ok(new_external_expense));
        external_expense_repo
            .expect_update()
            .once()
            .returning(|_| Ok(()));

        let external_expense_service = ExternalExpenseService {
            external_expense_repo,
        };

        let actual = external_expense_service
            .update_external_expense(external_expense.clone())
            .await
            .unwrap();
        assert_eq!(external_expense, actual);
    }

    #[tokio::test]
    async fn update_external_expense_failure_does_not_exist() {
        let external_expense = Faker.fake::<ExternalExpense>();
        let mut external_expense_repo = MockExternalExpenseRepo::new();
        external_expense_repo
            .expect_get()
            .return_once(|_| Err(AppError::ResourceNotFound));
        external_expense_repo.expect_update().never();

        let external_expense_service = ExternalExpenseService {
            external_expense_repo,
        };

        let actual = external_expense_service
            .update_external_expense(external_expense.clone())
            .await;
        assert!(matches!(actual, Err(AppError::ResourceNotFound)));
    }

    #[tokio::test]
    async fn delete_external_expense_success() {
        let external_expense = Faker.fake::<ExternalExpense>();
        let new_external_expense = external_expense.clone();
        let external_expense_id = external_expense.id;
        let mut external_expense_repo = MockExternalExpenseRepo::new();
        external_expense_repo
            .expect_get()
            .return_once(|_| Ok(new_external_expense));
        external_expense_repo
            .expect_delete()
            .once()
            .returning(|_| Ok(()));

        let external_expense_service = ExternalExpenseService {
            external_expense_repo,
        };

        let actual = external_expense_service
            .delete_external_expense(external_expense_id)
            .await
            .unwrap();
        assert_eq!(external_expense, actual);
    }

    #[tokio::test]
    async fn delete_external_expense_failure_does_not_exist() {
        let external_expense = Faker.fake::<ExternalExpense>();
        let external_expense_id = external_expense.id;
        let mut external_expense_repo = MockExternalExpenseRepo::new();
        external_expense_repo
            .expect_get()
            .return_once(|_| Err(AppError::ResourceNotFound));
        external_expense_repo.expect_delete().never();

        let external_expense_service = ExternalExpenseService {
            external_expense_repo,
        };

        let actual = external_expense_service
            .delete_external_expense(external_expense_id)
            .await;
        assert!(matches!(actual, Err(AppError::ResourceNotFound)));
    }
}
