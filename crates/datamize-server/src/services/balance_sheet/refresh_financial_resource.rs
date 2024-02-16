use std::{collections::HashSet, sync::Arc};

use chrono::{Datelike, Local};
use datamize_domain::{
    async_trait,
    db::{DbError, DynFinResRepo, DynMonthRepo, DynYearRepo},
    FinancialResourceYearly, Month, MonthNum, ResourcesToRefresh, Uuid, YearlyBalances,
};
use ynab::AccountRequests;

use crate::{error::DatamizeResult, services::budget_providers::DynExternalAccountService};

#[async_trait]
pub trait RefreshFinResServiceExt: Send + Sync {
    async fn refresh_fin_res(
        &self,
        resources_to_refresh: Option<ResourcesToRefresh>,
    ) -> DatamizeResult<Vec<Uuid>>;
}

pub type DynRefreshFinResService = Arc<dyn RefreshFinResServiceExt>;

#[derive(Clone)]
pub struct RefreshFinResService {
    pub fin_res_repo: DynFinResRepo,
    pub month_repo: DynMonthRepo,
    pub year_repo: DynYearRepo,
    pub external_account_service: DynExternalAccountService,
    pub ynab_client: Arc<dyn AccountRequests + Send + Sync>,
}

#[async_trait]
impl RefreshFinResServiceExt for RefreshFinResService {
    #[tracing::instrument(skip_all)]
    async fn refresh_fin_res(
        &self,
        resources_to_refresh: Option<ResourcesToRefresh>,
    ) -> DatamizeResult<Vec<Uuid>> {
        let current_date = Local::now().date_naive();
        let current_year = current_date.year();
        // The only condition is that the year exists...
        let mut year_data = self.year_repo.get_year_data_by_number(current_year).await?;

        let current_month: MonthNum = current_date.month().try_into().unwrap();
        self.ensure_month_exists(current_year, current_month)
            .await?;

        let mut resources = self.fin_res_repo.get_from_year(current_year).await?;
        resources.retain(|r| {
            resources_to_refresh
                .as_ref()
                .map_or(true, |refresh| refresh.ids.contains(&r.base.id))
        });

        let accounts = self.ynab_client.get_accounts().await?;
        let external_accounts = self
            .external_account_service
            .refresh_web_scraping_accounts(self.get_external_account_ids(&resources))
            .await?;

        let mut refreshed = HashSet::new();

        for res in &mut resources {
            if let Some(ref account_ids) = res.base.ynab_account_ids {
                let is_account_included = accounts
                    .iter()
                    .filter(|a| account_ids.contains(&a.id))
                    .count()
                    > 0;

                if !account_ids.is_empty() && is_account_included {
                    let balance = accounts
                        .iter()
                        .filter(|a| account_ids.contains(&a.id))
                        .map(|a| a.balance.abs())
                        .sum::<i64>();

                    match res.get_balance(current_year, current_month) {
                        Some(current_balance) => {
                            if current_balance != balance {
                                res.insert_balance(current_year, current_month, balance);
                                refreshed.insert(res.base.id);
                            }
                        }
                        None => {
                            res.insert_balance(current_year, current_month, balance);
                            refreshed.insert(res.base.id);
                        }
                    }
                }
            }

            if let Some(ref account_ids) = res.base.external_account_ids {
                let is_account_included = external_accounts
                    .iter()
                    .filter(|a| account_ids.contains(&a.id))
                    .count()
                    > 0;

                if !account_ids.is_empty() && is_account_included {
                    let balance = external_accounts
                        .iter()
                        .filter(|a| account_ids.contains(&a.id))
                        .map(|a| a.balance.abs())
                        .sum::<i64>();

                    match res.get_balance(current_year, current_month) {
                        Some(current_balance) => {
                            if current_balance != balance {
                                res.insert_balance(current_year, current_month, balance);
                                refreshed.insert(res.base.id);
                            }
                        }
                        None => {
                            res.insert_balance(current_year, current_month, balance);
                            refreshed.insert(res.base.id);
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

        Ok(refreshed.into_iter().collect())
    }
}

impl RefreshFinResService {
    pub fn new_arced(
        fin_res_repo: DynFinResRepo,
        month_repo: DynMonthRepo,
        year_repo: DynYearRepo,
        external_account_service: DynExternalAccountService,
        ynab_client: Arc<dyn AccountRequests + Send + Sync>,
    ) -> Arc<Self> {
        Arc::new(Self {
            year_repo,
            month_repo,
            fin_res_repo,
            external_account_service,
            ynab_client,
        })
    }

    /// Check if month exists, create if not
    async fn ensure_month_exists(&self, year: i32, month: MonthNum) -> DatamizeResult<()> {
        if let Err(DbError::NotFound) = self.month_repo.get_month_data_by_number(month, year).await
        {
            // If month doesn't exist, create it
            let month = Month::new(month, year);
            self.month_repo.add(&month, year).await?;
        }

        Ok(())
    }

    fn get_external_account_ids(&self, resources: &[FinancialResourceYearly]) -> Vec<Uuid> {
        resources
            .iter()
            .flat_map(|r| r.base.external_account_ids.as_ref())
            .flatten()
            .copied()
            .collect::<Vec<_>>()
    }
}
