use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    error::DatamizeResult,
    models::balance_sheet::{
        FinancialResourceMonthly, FinancialResourceYearly, Month, MonthNum, NetTotal, SavingRate,
        Year,
    },
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YearRepo: Send + Sync {
    async fn get_years(&self) -> DatamizeResult<Vec<Year>>;
    async fn get_year_data_by_number(&self, year: i32) -> DatamizeResult<YearData>;
    async fn add(&self, year: &Year) -> DatamizeResult<()>;
    async fn get(&self, year: i32) -> DatamizeResult<Year>;
    async fn get_net_totals(&self, year_id: Uuid) -> DatamizeResult<Vec<NetTotal>>;
    async fn update_net_totals(&self, year: i32) -> DatamizeResult<()>;
    async fn update_refreshed_at(&self, year: &YearData) -> DatamizeResult<()>;
    async fn delete(&self, year: i32) -> DatamizeResult<()>;
}

pub type DynYearRepo = Arc<dyn YearRepo>;

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct YearData {
    pub id: Uuid,
    pub year: i32,
    pub refreshed_at: DateTime<Utc>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MonthRepo: Send + Sync {
    async fn get_year_data_by_number(&self, year: i32) -> DatamizeResult<YearData>;
    async fn get_month_data_by_number(
        &self,
        month: MonthNum,
        year: i32,
    ) -> DatamizeResult<MonthData>;
    async fn get_months_of_year(&self, year: i32) -> DatamizeResult<Vec<Month>>;
    async fn get_months(&self) -> DatamizeResult<Vec<Month>>;
    async fn add(&self, month: &Month, year: i32) -> DatamizeResult<()>;
    async fn get(&self, month_num: MonthNum, year: i32) -> DatamizeResult<Month>;
    async fn get_net_totals(&self, month_id: Uuid) -> DatamizeResult<Vec<NetTotal>>;
    async fn update_net_totals(&self, month_num: MonthNum, year: i32) -> DatamizeResult<()>;
    async fn delete(&self, month_num: MonthNum, year: i32) -> DatamizeResult<()>;
}

pub type DynMonthRepo = Arc<dyn MonthRepo>;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MonthData {
    pub id: Uuid,
    pub month: i16,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FinResRepo: Send + Sync {
    async fn get_from_all_years(&self) -> DatamizeResult<Vec<FinancialResourceYearly>>;
    async fn get_from_year(&self, year: i32) -> DatamizeResult<Vec<FinancialResourceYearly>>;
    async fn get_from_month(
        &self,
        month: MonthNum,
        year: i32,
    ) -> DatamizeResult<Vec<FinancialResourceMonthly>>;
    async fn get(&self, resource_id: Uuid) -> DatamizeResult<FinancialResourceYearly>;
    async fn update(&self, resource: &FinancialResourceYearly) -> DatamizeResult<()>;
    async fn update_monthly(&self, resource: &FinancialResourceMonthly) -> DatamizeResult<()>;
    async fn delete(&self, resource_id: Uuid) -> DatamizeResult<()>;
}

pub type DynFinResRepo = Arc<dyn FinResRepo>;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SavingRateRepo: Send + Sync {
    async fn get_from_year(&self, year: i32) -> DatamizeResult<Vec<SavingRate>>;
    async fn get(&self, saving_rate_id: Uuid) -> DatamizeResult<SavingRate>;
    async fn get_by_name(&self, name: &str) -> DatamizeResult<SavingRate>;
    async fn update(&self, saving_rate: &SavingRate) -> DatamizeResult<()>;
    async fn delete(&self, saving_rate_id: Uuid) -> DatamizeResult<()>;
}

pub type DynSavingRateRepo = Arc<dyn SavingRateRepo>;
