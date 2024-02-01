use std::sync::Arc;

use async_trait::async_trait;
use datamize_domain::{
    db::{DbError, DynBudgeterConfigRepo},
    BudgeterConfig, SaveBudgeterConfig, Uuid,
};

use crate::error::{AppError, DatamizeResult};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BudgeterServiceExt: Send + Sync {
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

pub type DynBudgeterService = Arc<dyn BudgeterServiceExt>;

pub struct BudgeterService {
    pub budgeter_config_repo: DynBudgeterConfigRepo,
}

impl BudgeterService {
    pub fn new_arced(budgeter_config_repo: DynBudgeterConfigRepo) -> Arc<Self> {
        Arc::new(Self {
            budgeter_config_repo,
        })
    }
}

#[async_trait]
impl BudgeterServiceExt for BudgeterService {
    #[tracing::instrument(skip(self))]
    async fn get_all_budgeters(&self) -> DatamizeResult<Vec<BudgeterConfig>> {
        Ok(self.budgeter_config_repo.get_all().await?)
    }

    #[tracing::instrument(skip_all)]
    async fn create_budgeter(
        &self,
        new_budgeter: SaveBudgeterConfig,
    ) -> DatamizeResult<BudgeterConfig> {
        let Err(DbError::NotFound) = self
            .budgeter_config_repo
            .get_by_name(&new_budgeter.name)
            .await
        else {
            return Err(AppError::ResourceAlreadyExist);
        };

        let budgeter_config: BudgeterConfig = new_budgeter.into();
        self.budgeter_config_repo.update(&budgeter_config).await?;

        Ok(budgeter_config)
    }

    #[tracing::instrument(skip(self))]
    async fn get_budgeter(&self, budgeter_id: Uuid) -> DatamizeResult<BudgeterConfig> {
        Ok(self.budgeter_config_repo.get(budgeter_id).await?)
    }

    #[tracing::instrument(skip(self, new_budgeter))]
    async fn update_budgeter(
        &self,
        new_budgeter: BudgeterConfig,
    ) -> DatamizeResult<BudgeterConfig> {
        let Ok(_) = self.budgeter_config_repo.get(new_budgeter.id).await else {
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
