use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::balance_sheet::{
    FinancialResourceMonthly, FinancialResourceYearly, Month, MonthNum, NetTotal,
    SavingRatesPerPerson, YearDetail, YearSummary,
};

use super::postgres;

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct YearData {
    pub id: Uuid,
    pub year: i32,
    pub refreshed_at: DateTime<Utc>,
}

#[tracing::instrument(skip_all)]
pub async fn get_years_summary(db_conn_pool: &PgPool) -> Result<Vec<YearSummary>, sqlx::Error> {
    postgres::get_years_summary(db_conn_pool).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_year_data(db_conn_pool: &PgPool, year: i32) -> Result<YearData, sqlx::Error> {
    postgres::get_year_data(db_conn_pool, year).await
}

#[tracing::instrument(skip_all)]
pub async fn add_new_year(db_conn_pool: &PgPool, year: &YearDetail) -> Result<(), sqlx::Error> {
    postgres::add_new_year(db_conn_pool, year).await
}

#[tracing::instrument(skip(db_conn_pool, net_totals))]
pub async fn insert_yearly_net_totals(
    db_conn_pool: &PgPool,
    year_id: Uuid,
    net_totals: [&NetTotal; 2],
) -> Result<(), sqlx::Error> {
    postgres::insert_yearly_net_totals(db_conn_pool, year_id, net_totals).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_year_net_totals_for(
    db_conn_pool: &PgPool,
    year_id: Uuid,
) -> Result<Vec<NetTotal>, sqlx::Error> {
    postgres::get_year_net_totals_for(db_conn_pool, year_id).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_saving_rates_for(
    db_conn_pool: &PgPool,
    year_id: Uuid,
) -> Result<Vec<SavingRatesPerPerson>, sqlx::Error> {
    postgres::get_saving_rates_for(db_conn_pool, year_id).await
}

#[tracing::instrument(skip_all)]
pub async fn update_saving_rates(
    db_conn_pool: &PgPool,
    year: &YearDetail,
) -> Result<(), sqlx::Error> {
    postgres::update_saving_rates(db_conn_pool, year).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn delete_year(db_conn_pool: &PgPool, year: i32) -> Result<(), sqlx::Error> {
    postgres::delete_year(db_conn_pool, year).await
}

#[derive(sqlx::FromRow, Debug)]
pub struct MonthData {
    pub id: Uuid,
    pub month: i16,
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_month_data(
    db_conn_pool: &PgPool,
    month: MonthNum,
    year: i32,
) -> Result<MonthData, sqlx::Error> {
    postgres::get_month_data(db_conn_pool, month, year).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_months(db_conn_pool: &PgPool, year: i32) -> Result<Vec<Month>, sqlx::Error> {
    postgres::get_months(db_conn_pool, year).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_all_months(db_conn_pool: &PgPool) -> Result<Vec<Month>, sqlx::Error> {
    postgres::get_all_months(db_conn_pool).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_month(
    db_conn_pool: &PgPool,
    month_num: MonthNum,
    year: i32,
) -> Result<Month, sqlx::Error> {
    postgres::get_month(db_conn_pool, month_num, year).await
}

#[tracing::instrument(skip_all)]
pub async fn add_new_month(
    db_conn_pool: &PgPool,
    month: &Month,
    year: i32,
) -> Result<(), sqlx::Error> {
    postgres::add_new_month(db_conn_pool, month, year).await
}

#[tracing::instrument(skip(db_conn_pool, net_totals))]
pub async fn insert_monthly_net_totals(
    db_conn_pool: &PgPool,
    month_id: Uuid,
    net_totals: [&NetTotal; 2],
) -> Result<(), sqlx::Error> {
    postgres::insert_monthly_net_totals(db_conn_pool, month_id, net_totals).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_month_net_totals_for(
    db_conn_pool: &PgPool,
    month_id: Uuid,
) -> Result<Vec<NetTotal>, sqlx::Error> {
    postgres::get_month_net_totals_for(db_conn_pool, month_id).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_months_of_resource(
    db_conn_pool: &PgPool,
    resource_id: Uuid,
) -> Result<Vec<MonthData>, sqlx::Error> {
    postgres::get_months_of_resource(db_conn_pool, resource_id).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn delete_month(
    db_conn_pool: &PgPool,
    month_num: MonthNum,
    year: i32,
) -> Result<(), sqlx::Error> {
    postgres::delete_month(db_conn_pool, month_num, year).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_all_financial_resources_of_all_years(
    db_conn_pool: &PgPool,
) -> Result<Vec<FinancialResourceYearly>, sqlx::Error> {
    postgres::get_all_financial_resources_of_all_years(db_conn_pool).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_financial_resources_of_year(
    db_conn_pool: &PgPool,
    year: i32,
) -> Result<Vec<FinancialResourceYearly>, sqlx::Error> {
    postgres::get_financial_resources_of_year(db_conn_pool, year).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_financial_resources_of_month(
    db_conn_pool: &PgPool,
    month: MonthNum,
    year: i32,
) -> Result<Vec<FinancialResourceMonthly>, sqlx::Error> {
    postgres::get_financial_resources_of_month(db_conn_pool, month, year).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_financial_resource(
    db_conn_pool: &PgPool,
    resource_id: Uuid,
) -> Result<FinancialResourceYearly, sqlx::Error> {
    postgres::get_financial_resource(db_conn_pool, resource_id).await
}

#[tracing::instrument(skip_all)]
pub async fn update_financial_resource(
    db_conn_pool: &PgPool,
    resource: &FinancialResourceYearly,
) -> Result<(), sqlx::Error> {
    postgres::update_financial_resource(db_conn_pool, resource).await
}

#[tracing::instrument(skip_all)]
pub async fn update_monthly_financial_resource(
    db_conn_pool: &PgPool,
    resource: &FinancialResourceMonthly,
) -> Result<(), sqlx::Error> {
    postgres::update_monthly_financial_resource(db_conn_pool, resource).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn delete_financial_resource(
    db_conn_pool: &PgPool,
    resource_id: Uuid,
) -> Result<(), sqlx::Error> {
    postgres::delete_financial_resource(db_conn_pool, resource_id).await
}
