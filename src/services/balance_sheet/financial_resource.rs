use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Datelike, Local};
use uuid::Uuid;
use ynab::AccountRequests;

use crate::{
    db::balance_sheet::{FinResRepo, MonthRepo, YearRepo},
    error::{AppError, DatamizeResult},
    models::balance_sheet::{FinancialResourceYearly, Month, MonthNum, SaveResource},
    services::budget_providers::ExternalAccountServiceExt,
    telemetry::spawn_blocking_with_tracing,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FinResServiceExt {
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
    async fn refresh_fin_res<EAS: ExternalAccountServiceExt + Send + Sync + 'static>(
        &self,
        mut external_acount_service: EAS,
        ynab_client: Arc<dyn AccountRequests + Send + Sync>,
    ) -> DatamizeResult<Vec<Uuid>>;
}

pub struct FinResService {
    pub fin_res_repo: Box<dyn FinResRepo + Sync + Send>,
    pub month_repo: Box<dyn MonthRepo + Sync + Send>,
    pub year_repo: Box<dyn YearRepo + Sync + Send>,
}

#[async_trait]
impl FinResServiceExt for FinResService {
    #[tracing::instrument(skip(self))]
    async fn get_all_fin_res(&self) -> DatamizeResult<Vec<FinancialResourceYearly>> {
        self.fin_res_repo.get_from_all_years().await
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_fin_res_from_year(
        &self,
        year: i32,
    ) -> DatamizeResult<Vec<FinancialResourceYearly>> {
        self.fin_res_repo.get_from_year(year).await
    }

    #[tracing::instrument(skip_all)]
    async fn create_fin_res(
        &self,
        new_fin_res: SaveResource,
    ) -> DatamizeResult<FinancialResourceYearly> {
        let resource: FinancialResourceYearly = new_fin_res.into();

        if !resource.balance_per_month.is_empty() {
            for month in resource.balance_per_month.keys() {
                if let Err(AppError::ResourceNotFound) = self
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
        self.fin_res_repo.get(fin_res_id).await
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
                if let Err(AppError::ResourceNotFound) = self
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

    #[tracing::instrument(skip_all)]
    async fn refresh_fin_res<EAS: ExternalAccountServiceExt + Send + Sync + 'static>(
        &self,
        mut external_acount_service: EAS,
        ynab_client: Arc<dyn AccountRequests + Send + Sync>,
    ) -> DatamizeResult<Vec<Uuid>> {
        let current_date = Local::now().date_naive();
        let current_year = current_date.year();
        // The only condition is that the year exists...
        let mut year_data = self.year_repo.get_year_data_by_number(current_year).await?;

        let current_month: MonthNum = current_date.month().try_into().unwrap();
        if let Err(AppError::ResourceNotFound) = self
            .month_repo
            .get_month_data_by_number(current_month, current_year)
            .await
        {
            // If month doesn't exist, create it
            let month = Month::new(current_month, current_year);
            self.month_repo.add(&month, current_year).await?;
        }

        let mut resources = self.fin_res_repo.get_from_year(current_year).await?;

        let accounts = ynab_client.get_accounts().await?;
        let external_accounts = spawn_blocking_with_tracing(move || async move {
            external_acount_service
                .refresh_all_web_scraping_accounts()
                .await
        })
        .await
        .unwrap()
        .await?;

        let mut refreshed = vec![];

        for res in &mut resources {
            if let Some(ref account_ids) = res.base.ynab_account_ids {
                if !account_ids.is_empty() {
                    let balance = accounts
                        .iter()
                        .filter(|a| account_ids.contains(&a.id))
                        .map(|a| a.balance.abs())
                        .sum::<i64>();

                    match res.balance_per_month.get_mut(&current_month) {
                        Some(current_balance) => {
                            if *current_balance != balance {
                                *current_balance = balance;
                                refreshed.push(res.base.id);
                            }
                        }
                        None => {
                            res.balance_per_month.insert(current_month, balance);
                            refreshed.push(res.base.id);
                        }
                    }
                }
            }

            if let Some(ref account_ids) = res.base.external_account_ids {
                if !account_ids.is_empty() {
                    let balance = external_accounts
                        .iter()
                        .filter(|a| account_ids.contains(&a.id))
                        .map(|a| a.balance.abs())
                        .sum::<i64>();

                    match res.balance_per_month.get_mut(&current_month) {
                        Some(current_balance) => {
                            if *current_balance != balance {
                                *current_balance = balance;
                                refreshed.push(res.base.id);
                            }
                        }
                        None => {
                            res.balance_per_month.insert(current_month, balance);
                            refreshed.push(res.base.id);
                        }
                    }
                }
            }
        }

        if !refreshed.is_empty() {
            year_data.refreshed_at = chrono::Utc::now();
            self.year_repo.update_refreshed_at(&year_data).await?;
            resources.retain(|r| refreshed.contains(&r.base.id));
            for r in resources {
                self.fin_res_repo.update(&r).await?;
            }
            self.month_repo
                .update_net_totals(current_month, current_year)
                .await?;
            self.year_repo.update_net_totals(current_year).await?;
        }

        Ok(refreshed)
    }
}
