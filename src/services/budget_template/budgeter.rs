use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    db::budget_template::BudgeterConfigRepo,
    error::{AppError, DatamizeResult},
    models::budget_template::{BudgeterConfig, SaveBudgeterConfig},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BudgeterServiceExt {
    async fn get_all_budgeters(&self) -> DatamizeResult<Vec<BudgeterConfig>>;
    async fn create_budgeter(
        &self,
        new_budgeter: SaveBudgeterConfig,
    ) -> DatamizeResult<BudgeterConfig>;
    async fn get_budgeter(&self, budgeter_id: Uuid) -> DatamizeResult<BudgeterConfig>;
    async fn update_budgeter(&self, new_budgeter: BudgeterConfig)
        -> DatamizeResult<BudgeterConfig>;
    async fn delete_budgeter(&self, budgeter_id: Uuid) -> DatamizeResult<BudgeterConfig>;
}

pub struct BudgeterService<BCR: BudgeterConfigRepo> {
    pub budgeter_config_repo: BCR,
}

#[async_trait]
impl<BCR> BudgeterServiceExt for BudgeterService<BCR>
where
    BCR: BudgeterConfigRepo + Sync + Send,
{
    #[tracing::instrument(skip(self))]
    async fn get_all_budgeters(&self) -> DatamizeResult<Vec<BudgeterConfig>> {
        self.budgeter_config_repo.get_all().await
    }

    #[tracing::instrument(skip_all)]
    async fn create_budgeter(
        &self,
        new_budgeter: SaveBudgeterConfig,
    ) -> DatamizeResult<BudgeterConfig> {
        let Err(AppError::ResourceNotFound) = self.budgeter_config_repo.get_by_name(&new_budgeter.name).await else
        {
            return Err(AppError::ResourceAlreadyExist);
        };

        let budgeter_config: BudgeterConfig = new_budgeter.into();
        self.budgeter_config_repo.update(&budgeter_config).await?;

        Ok(budgeter_config)
    }

    #[tracing::instrument(skip(self))]
    async fn get_budgeter(&self, budgeter_id: Uuid) -> DatamizeResult<BudgeterConfig> {
        self.budgeter_config_repo.get(budgeter_id).await
    }

    #[tracing::instrument(skip(self, new_budgeter))]
    async fn update_budgeter(
        &self,
        new_budgeter: BudgeterConfig,
    ) -> DatamizeResult<BudgeterConfig> {
        let Ok(_) =  self.budgeter_config_repo.get(new_budgeter.id).await else {
                return Err(AppError::ResourceNotFound);
            };

        self.budgeter_config_repo.update(&new_budgeter).await?;

        Ok(new_budgeter)
    }

    #[tracing::instrument(skip(self))]
    async fn delete_budgeter(&self, budgeter_id: Uuid) -> DatamizeResult<BudgeterConfig> {
        let Ok(budgeter_config) = self.budgeter_config_repo.get(budgeter_id).await else {
            return Err(AppError::ResourceNotFound);
        };

        self.budgeter_config_repo.delete(budgeter_id).await?;

        Ok(budgeter_config)
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;
    use crate::db::budget_template::MockBudgeterConfigRepo;

    #[tokio::test]
    async fn create_budgeter_success() {
        let mut budgeter_config_repo = MockBudgeterConfigRepo::new();
        budgeter_config_repo
            .expect_get_by_name()
            .once()
            .returning(|_| Err(AppError::ResourceNotFound));
        budgeter_config_repo
            .expect_update()
            .once()
            .returning(|_| Ok(()));

        let budgeter_service = BudgeterService {
            budgeter_config_repo,
        };

        let new_budgeter: SaveBudgeterConfig = Faker.fake();

        let expected: BudgeterConfig = new_budgeter.clone().into();
        let actual = budgeter_service
            .create_budgeter(new_budgeter)
            .await
            .unwrap();
        assert_eq!(expected.name, actual.name);
        assert_eq!(expected.payee_ids, actual.payee_ids);
    }

    #[tokio::test]
    async fn create_budgeter_failure_already_exist() {
        let mut budgeter_config_repo = MockBudgeterConfigRepo::new();
        budgeter_config_repo
            .expect_get_by_name()
            .once()
            .returning(|_| Ok(Faker.fake::<BudgeterConfig>()));
        budgeter_config_repo.expect_update().never();

        let budgeter_service = BudgeterService {
            budgeter_config_repo,
        };

        let new_budgeter: SaveBudgeterConfig = Faker.fake();

        let actual = budgeter_service.create_budgeter(new_budgeter).await;
        assert!(matches!(actual, Err(AppError::ResourceAlreadyExist)));
    }

    #[tokio::test]
    async fn update_budgeter_success() {
        let budgeter_config = Faker.fake::<BudgeterConfig>();
        let new_budgeter = budgeter_config.clone();
        let mut budgeter_config_repo = MockBudgeterConfigRepo::new();
        budgeter_config_repo
            .expect_get()
            .return_once(|_| Ok(new_budgeter));
        budgeter_config_repo
            .expect_update()
            .once()
            .returning(|_| Ok(()));

        let budgeter_service = BudgeterService {
            budgeter_config_repo,
        };

        let actual = budgeter_service
            .update_budgeter(budgeter_config.clone())
            .await
            .unwrap();
        assert_eq!(budgeter_config, actual);
    }

    #[tokio::test]
    async fn update_budgeter_failure_does_not_exist() {
        let budgeter_config = Faker.fake::<BudgeterConfig>();
        let mut budgeter_config_repo = MockBudgeterConfigRepo::new();
        budgeter_config_repo
            .expect_get()
            .return_once(|_| Err(AppError::ResourceNotFound));
        budgeter_config_repo.expect_update().never();

        let budgeter_service = BudgeterService {
            budgeter_config_repo,
        };

        let actual = budgeter_service
            .update_budgeter(budgeter_config.clone())
            .await;
        assert!(matches!(actual, Err(AppError::ResourceNotFound)));
    }

    #[tokio::test]
    async fn delete_budgeter_success() {
        let budgeter_config = Faker.fake::<BudgeterConfig>();
        let new_budgeter = budgeter_config.clone();
        let budgeter_config_id = budgeter_config.id;
        let mut budgeter_config_repo = MockBudgeterConfigRepo::new();
        budgeter_config_repo
            .expect_get()
            .return_once(|_| Ok(new_budgeter));
        budgeter_config_repo
            .expect_delete()
            .once()
            .returning(|_| Ok(()));

        let budgeter_service = BudgeterService {
            budgeter_config_repo,
        };

        let actual = budgeter_service
            .delete_budgeter(budgeter_config_id)
            .await
            .unwrap();
        assert_eq!(budgeter_config, actual);
    }

    #[tokio::test]
    async fn delete_budgeter_failure_does_not_exist() {
        let budgeter_config = Faker.fake::<BudgeterConfig>();
        let budgeter_config_id = budgeter_config.id;
        let mut budgeter_config_repo = MockBudgeterConfigRepo::new();
        budgeter_config_repo
            .expect_get()
            .return_once(|_| Err(AppError::ResourceNotFound));
        budgeter_config_repo.expect_delete().never();

        let budgeter_service = BudgeterService {
            budgeter_config_repo,
        };

        let actual = budgeter_service.delete_budgeter(budgeter_config_id).await;
        assert!(matches!(actual, Err(AppError::ResourceNotFound)));
    }
}
