use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Datelike, Local};
use dyn_clone::{clone_trait_object, DynClone};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;
use ynab::AccountRequests;

use crate::{
    db::balance_sheet::{
        FinResRepo, MonthRepo, PostgresFinResRepo, PostgresMonthRepo, PostgresYearRepo, YearRepo,
    },
    error::{AppError, DatamizeResult},
    models::balance_sheet::{Month, MonthNum},
    services::budget_providers::{DynExternalAccountService, ExternalAccountService},
    telemetry::spawn_blocking_with_tracing,
};

#[async_trait]
pub trait RefreshFinResServiceExt: DynClone {
    async fn refresh_fin_res(&mut self) -> DatamizeResult<Vec<Uuid>>;
}

clone_trait_object!(RefreshFinResServiceExt);

pub type DynRefreshFinResService = Box<dyn RefreshFinResServiceExt + Send + Sync>;

#[derive(Clone)]
pub struct RefreshFinResService {
    pub fin_res_repo: Arc<dyn FinResRepo + Sync + Send>,
    pub month_repo: Arc<dyn MonthRepo + Sync + Send>,
    pub year_repo: Arc<dyn YearRepo + Sync + Send>,
    pub external_account_service: DynExternalAccountService,
    pub ynab_client: Arc<dyn AccountRequests + Send + Sync>,
}

#[async_trait]
impl RefreshFinResServiceExt for RefreshFinResService {
    #[tracing::instrument(skip_all)]
    async fn refresh_fin_res(&mut self) -> DatamizeResult<Vec<Uuid>> {
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

        let accounts = self.ynab_client.get_accounts().await?;
        let mut external_account_service = self.external_account_service.clone();
        let external_accounts = spawn_blocking_with_tracing(move || async move {
            external_account_service
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

impl RefreshFinResService {
    pub fn new_boxed(
        db_conn_pool: PgPool,
        redis_conn: ConnectionManager,
        ynab_client: Arc<dyn AccountRequests + Send + Sync>,
    ) -> Box<Self> {
        Box::new(Self {
            year_repo: PostgresYearRepo::new_arced(db_conn_pool.clone()),
            month_repo: PostgresMonthRepo::new_arced(db_conn_pool.clone()),
            fin_res_repo: PostgresFinResRepo::new_arced(db_conn_pool.clone()),
            external_account_service: ExternalAccountService::new_boxed(db_conn_pool, redis_conn),
            ynab_client,
        })
    }
}
