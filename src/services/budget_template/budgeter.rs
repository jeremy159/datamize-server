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
