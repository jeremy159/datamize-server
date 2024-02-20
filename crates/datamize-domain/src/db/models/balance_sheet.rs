use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    db::error::DbResult,
    models::{
        FinancialResourceMonthly, FinancialResourceYearly, Month, MonthNum, SavingRate, Year,
    },
    NetTotals, UpdateResource,
};

#[async_trait]
pub trait YearRepo: Send + Sync {
    async fn get_years(&self) -> DbResult<Vec<Year>>;
    async fn get_years_starting_from(&self, year: i32) -> DbResult<Vec<Year>>;
    async fn get_year_data_by_number(&self, year: i32) -> DbResult<YearData>;
    async fn add(&self, year: &Year) -> DbResult<()>;
    async fn get_without_resources(&self, year: i32) -> DbResult<Year>;
    async fn get(&self, year: i32) -> DbResult<Year>;
    async fn get_net_totals(&self, year_id: Uuid) -> DbResult<NetTotals>;
    async fn update_net_totals(&self, year: i32) -> DbResult<()>;
    async fn insert_net_totals(&self, year_id: Uuid, net_totals: &NetTotals) -> DbResult<()>;
    async fn update_refreshed_at(&self, year: &YearData) -> DbResult<()>;
    async fn delete(&self, year: i32) -> DbResult<()>;
}

pub type DynYearRepo = Arc<dyn YearRepo>;

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct YearData {
    pub id: Uuid,
    pub year: i32,
    pub refreshed_at: DateTime<Utc>,
}

#[async_trait]
pub trait MonthRepo: Send + Sync {
    async fn get_year_data_by_number(&self, year: i32) -> DbResult<YearData>;
    async fn get_month_data_by_number(&self, month: MonthNum, year: i32) -> DbResult<MonthData>;
    async fn get_months_of_year_without_resources(&self, year: i32) -> DbResult<Vec<Month>>;
    async fn get_months_of_year(&self, year: i32) -> DbResult<Vec<Month>>;
    async fn get_months(&self) -> DbResult<Vec<Month>>;
    async fn get_months_starting_from(
        &self,
        month_num: MonthNum,
        year: i32,
    ) -> DbResult<Vec<Month>>;
    async fn add(&self, month: &Month, year: i32) -> DbResult<()>;
    async fn get_without_resources(&self, month_num: MonthNum, year: i32) -> DbResult<Month>;
    async fn get(&self, month_num: MonthNum, year: i32) -> DbResult<Month>;
    async fn get_net_totals(&self, month_id: Uuid) -> DbResult<NetTotals>;
    async fn update_net_totals(&self, month_num: MonthNum, year: i32) -> DbResult<()>;
    async fn insert_net_totals(&self, month_id: Uuid, net_totals: &NetTotals) -> DbResult<()>;
    async fn delete(&self, month_num: MonthNum, year: i32) -> DbResult<()>;
}

pub type DynMonthRepo = Arc<dyn MonthRepo>;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MonthData {
    pub id: Uuid,
    pub month: MonthNum,
    pub year: i32,
}

#[async_trait]
pub trait FinResRepo: Send + Sync {
    async fn get_from_all_years(&self) -> DbResult<Vec<FinancialResourceYearly>>;
    async fn get_from_year(&self, year: i32) -> DbResult<Vec<FinancialResourceYearly>>;
    async fn get_from_month(
        &self,
        month: MonthNum,
        year: i32,
    ) -> DbResult<Vec<FinancialResourceMonthly>>;
    async fn get(&self, resource_id: Uuid) -> DbResult<FinancialResourceYearly>;
    async fn get_by_name(&self, name: &str) -> DbResult<FinancialResourceYearly>;
    async fn update(&self, resource: &FinancialResourceYearly) -> DbResult<()>;
    async fn update_and_delete(&self, resource: &UpdateResource) -> DbResult<()>;
    async fn delete(&self, resource_id: Uuid) -> DbResult<()>;
}

pub type DynFinResRepo = Arc<dyn FinResRepo>;

#[async_trait]
pub trait SavingRateRepo: Send + Sync {
    async fn get_from_year(&self, year: i32) -> DbResult<Vec<SavingRate>>;
    async fn get(&self, saving_rate_id: Uuid) -> DbResult<SavingRate>;
    async fn get_by_name(&self, name: &str) -> DbResult<SavingRate>;
    async fn update(&self, saving_rate: &SavingRate) -> DbResult<()>;
    async fn delete(&self, saving_rate_id: Uuid) -> DbResult<()>;
}

pub type DynSavingRateRepo = Arc<dyn SavingRateRepo>;

#[derive(Debug, PartialEq, Clone, sqlx::Type)]
#[sqlx(type_name = "net_type")]
#[sqlx(rename_all = "camelCase")]
/// Type useful to know which kind of net total is stored in the database. Should not be used elsewhere.
pub enum NetTotalType {
    /// Net Assets is the total of owned assets minus the total of liabilities.
    Asset,
    /// Net Portfolio is the total of owned assets that are tangible cash. For example, bank or investments accounts
    /// are tangible cash assets but not the value of your house or car.
    Portfolio,
}

impl std::str::FromStr for NetTotalType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asset" => Ok(Self::Asset),
            "portfolio" => Ok(Self::Portfolio),
            _ => Err(format!("Failed to parse {:?} to NetTotalType", s)),
        }
    }
}

impl std::fmt::Display for NetTotalType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NetTotalType::Asset => write!(f, "asset"),
            NetTotalType::Portfolio => write!(f, "portfolio"),
        }
    }
}
