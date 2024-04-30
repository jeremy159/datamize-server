use std::{collections::HashSet, sync::Arc};

use datamize_domain::{
    async_trait,
    db::{DbError, DynFinResOrderRepo, DynFinResRepo, DynMonthRepo, DynYearRepo},
    FinancialResourceYearly, Month, MonthNum, ResourceCategory, SaveResource, Uuid, Year,
    YearlyBalances,
};

use crate::error::{AppError, DatamizeResult};

#[async_trait]
pub trait FinResServiceExt: Send + Sync {
    async fn get_all_fin_res(&self) -> DatamizeResult<Vec<FinancialResourceYearly>>;
    async fn get_all_fin_res_from_year(
        &self,
        year: i32,
    ) -> DatamizeResult<Vec<FinancialResourceYearly>>;
    async fn get_from_year_and_category(
        &self,
        year: i32,
        category: &ResourceCategory,
    ) -> DatamizeResult<Vec<FinancialResourceYearly>>;
    async fn save_resources_order(
        &self,
        year: i32,
        category: &ResourceCategory,
        order: &[Uuid],
    ) -> DatamizeResult<()>;
    async fn create_fin_res(
        &self,
        new_fin_res: SaveResource,
    ) -> DatamizeResult<FinancialResourceYearly>;
    async fn get_fin_res(&self, fin_res_id: Uuid) -> DatamizeResult<FinancialResourceYearly>;
    async fn update_fin_res(
        &self,
        new_fin_res: FinancialResourceYearly,
    ) -> DatamizeResult<FinancialResourceYearly>;
    async fn delete_fin_res(&self, fin_res_id: Uuid) -> DatamizeResult<FinancialResourceYearly>;
}

pub type DynFinResService = Arc<dyn FinResServiceExt>;

pub struct FinResService {
    pub fin_res_repo: DynFinResRepo,
    pub month_repo: DynMonthRepo,
    pub year_repo: DynYearRepo,
    pub fin_res_order_repo: DynFinResOrderRepo,
}

#[async_trait]
impl FinResServiceExt for FinResService {
    #[tracing::instrument(skip(self))]
    async fn get_all_fin_res(&self) -> DatamizeResult<Vec<FinancialResourceYearly>> {
        Ok(self.fin_res_repo.get_from_all_years().await?)
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_fin_res_from_year(
        &self,
        year: i32,
    ) -> DatamizeResult<Vec<FinancialResourceYearly>> {
        Ok(self.fin_res_repo.get_from_year(year).await?)
    }

    #[tracing::instrument(skip(self))]
    async fn get_from_year_and_category(
        &self,
        year: i32,
        category: &ResourceCategory,
    ) -> DatamizeResult<Vec<FinancialResourceYearly>> {
        let resources = self
            .fin_res_repo
            .get_from_year_and_category(year, category)
            .await?;
        let order = self.fin_res_order_repo.get_order(year, category).await?;
        let mut indexed_resources: Vec<_> = resources.iter().enumerate().collect();
        indexed_resources.sort_by_key(|(_, res)| {
            order
                .iter()
                .position(|&id| id == res.base.id)
                .unwrap_or(usize::MAX)
        });

        let resources: Vec<_> = indexed_resources
            .into_iter()
            .map(|(_, res)| res.clone())
            .collect();

        Ok(resources)
    }

    #[tracing::instrument(skip(self))]
    async fn save_resources_order(
        &self,
        year: i32,
        category: &ResourceCategory,
        order: &[Uuid],
    ) -> DatamizeResult<()> {
        self.fin_res_order_repo
            .set_order(year, category, order)
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn create_fin_res(
        &self,
        new_fin_res: SaveResource,
    ) -> DatamizeResult<FinancialResourceYearly> {
        let resource: FinancialResourceYearly = new_fin_res.into();

        let Err(DbError::NotFound) = self.fin_res_repo.get_by_name(&resource.base.name).await
        else {
            return Err(AppError::ResourceAlreadyExist);
        };

        self.ensure_month_year_exist(&resource).await?;
        self.fin_res_repo.update(&resource).await?;
        self.update_net_totals(resource.get_first_month()).await?;

        Ok(resource)
    }

    #[tracing::instrument(skip(self))]
    async fn get_fin_res(&self, fin_res_id: Uuid) -> DatamizeResult<FinancialResourceYearly> {
        Ok(self.fin_res_repo.get(fin_res_id).await?)
    }

    #[tracing::instrument(skip(self, updated_res))]
    async fn update_fin_res(
        &self,
        updated_res: FinancialResourceYearly,
    ) -> DatamizeResult<FinancialResourceYearly> {
        self.fin_res_repo.get(updated_res.base.id).await?;
        self.ensure_month_year_exist(&updated_res).await?;
        self.fin_res_repo.update_and_delete(&updated_res).await?;
        let resource = self.fin_res_repo.get(updated_res.base.id).await?;
        self.update_net_totals(updated_res.get_first_month())
            .await?;

        Ok(resource)
    }

    #[tracing::instrument(skip(self))]
    async fn delete_fin_res(&self, fin_res_id: Uuid) -> DatamizeResult<FinancialResourceYearly> {
        let resource = self.fin_res_repo.get(fin_res_id).await?;
        self.fin_res_repo.delete(fin_res_id).await?;
        self.update_net_totals(resource.get_first_month()).await?;

        Ok(resource)
    }
}

impl FinResService {
    pub fn new_arced(
        fin_res_repo: DynFinResRepo,
        month_repo: DynMonthRepo,
        year_repo: DynYearRepo,
        fin_res_order_repo: DynFinResOrderRepo,
    ) -> Arc<Self> {
        Arc::new(Self {
            year_repo,
            month_repo,
            fin_res_repo,
            fin_res_order_repo,
        })
    }

    async fn ensure_month_year_exist<T: YearlyBalances>(&self, resource: &T) -> DatamizeResult<()> {
        let mut checked_years = HashSet::<i32>::new();

        for (year, month, _) in resource.iter_all_balances() {
            if !checked_years.contains(&year) {
                checked_years.insert(year);
                if let Err(DbError::NotFound) = self.year_repo.get_year_data_by_number(year).await {
                    // If year doesn't exist, create it
                    let year = Year::new(year);
                    self.year_repo.add(&year).await?;
                }
            }

            if let Err(DbError::NotFound) =
                self.month_repo.get_month_data_by_number(month, year).await
            {
                // If month doesn't exist, create it
                let month = Month::new(month, year);
                self.month_repo.add(&month, year).await?;
            }
        }

        Ok(())
    }

    async fn update_net_totals(&self, first_month: Option<(i32, MonthNum)>) -> DatamizeResult<()> {
        if let Some((year, month)) = first_month {
            self.month_repo.update_net_totals(month, year).await?;
            self.year_repo.update_net_totals(year).await?;
        }

        Ok(())
    }
}
