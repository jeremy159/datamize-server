use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{DbError, DynFinResRepo, DynMonthRepo, DynYearRepo},
    FinancialResourceYearly, Month, SaveResource, Uuid,
};

use crate::error::DatamizeResult;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FinResServiceExt: Send + Sync {
    async fn get_all_fin_res(&self) -> DatamizeResult<Vec<FinancialResourceYearly>>;
    async fn get_all_fin_res_from_year(
        &self,
        year: i32,
    ) -> DatamizeResult<Vec<FinancialResourceYearly>>;
    async fn create_fin_res(
        &self,
        new_fin_res: SaveResource,
    ) -> DatamizeResult<FinancialResourceYearly>;
    async fn get_fin_res(&self, fin_res_id: Uuid) -> DatamizeResult<FinancialResourceYearly>;
    async fn update_fin_res(
        &self,
        fin_res_id: Uuid,
        new_fin_res: SaveResource,
    ) -> DatamizeResult<FinancialResourceYearly>;
    async fn delete_fin_res(&self, fin_res_id: Uuid) -> DatamizeResult<FinancialResourceYearly>;
}

pub type DynFinResService = Arc<dyn FinResServiceExt>;

pub struct FinResService {
    pub fin_res_repo: DynFinResRepo,
    pub month_repo: DynMonthRepo,
    pub year_repo: DynYearRepo,
}

impl FinResService {
    pub fn new_arced(
        fin_res_repo: DynFinResRepo,
        month_repo: DynMonthRepo,
        year_repo: DynYearRepo,
    ) -> Arc<Self> {
        Arc::new(Self {
            year_repo,
            month_repo,
            fin_res_repo,
        })
    }
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

    #[tracing::instrument(skip_all)]
    async fn create_fin_res(
        &self,
        new_fin_res: SaveResource,
    ) -> DatamizeResult<FinancialResourceYearly> {
        let resource: FinancialResourceYearly = new_fin_res.into();

        if !resource.balance_per_month.is_empty() {
            for month in resource.balance_per_month.keys() {
                if let Err(DbError::NotFound) = self
                    .month_repo
                    .get_month_data_by_number(*month, resource.year)
                    .await
                {
                    // If month doesn't exist, create it
                    let month = Month::new(*month, resource.year);
                    self.month_repo.add(&month, resource.year).await?;
                }
            }
        }

        self.fin_res_repo.update(&resource).await?;

        // If balance data was received, update month and year net totals
        if !resource.balance_per_month.is_empty() {
            self.month_repo
                .update_net_totals(
                    *resource.balance_per_month.first_key_value().unwrap().0,
                    resource.year,
                )
                .await?;
        }

        self.year_repo.update_net_totals(resource.year).await?;

        Ok(resource)
    }

    #[tracing::instrument(skip(self))]
    async fn get_fin_res(&self, fin_res_id: Uuid) -> DatamizeResult<FinancialResourceYearly> {
        Ok(self.fin_res_repo.get(fin_res_id).await?)
    }

    #[tracing::instrument(skip(self, new_fin_res))]
    async fn update_fin_res(
        &self,
        fin_res_id: Uuid,
        new_fin_res: SaveResource,
    ) -> DatamizeResult<FinancialResourceYearly> {
        let mut resource: FinancialResourceYearly = new_fin_res.into();
        resource.base.id = fin_res_id;

        self.fin_res_repo.get(fin_res_id).await?;

        if !resource.balance_per_month.is_empty() {
            for month in resource.balance_per_month.keys() {
                if let Err(DbError::NotFound) = self
                    .month_repo
                    .get_month_data_by_number(*month, resource.year)
                    .await
                {
                    // If month doesn't exist, create it
                    let month = Month::new(*month, resource.year);
                    self.month_repo.add(&month, resource.year).await?;
                }
            }
        }

        self.fin_res_repo.update(&resource).await?;

        // If balance data was received, update month and year net totals
        if !resource.balance_per_month.is_empty() {
            self.month_repo
                .update_net_totals(
                    *resource.balance_per_month.first_key_value().unwrap().0,
                    resource.year,
                )
                .await?;
        }

        self.year_repo.update_net_totals(resource.year).await?;

        Ok(resource)
    }

    #[tracing::instrument(skip(self))]
    async fn delete_fin_res(&self, fin_res_id: Uuid) -> DatamizeResult<FinancialResourceYearly> {
        let resource = self.fin_res_repo.get(fin_res_id).await?;
        self.fin_res_repo.delete(fin_res_id).await?;

        if !resource.balance_per_month.is_empty() {
            self.month_repo
                .update_net_totals(
                    *resource.balance_per_month.first_key_value().unwrap().0,
                    resource.year,
                )
                .await?;
        }

        self.year_repo.update_net_totals(resource.year).await?;

        Ok(resource)
    }
}
