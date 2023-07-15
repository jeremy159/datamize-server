use sqlx::PgPool;
use uuid::Uuid;

use crate::models::budget_template::BudgeterConfig;

use super::postgres;

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_all_budgeters_config(
    db_conn_pool: &PgPool,
) -> Result<Vec<BudgeterConfig>, sqlx::Error> {
    postgres::get_all_budgeters_config(db_conn_pool).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_budgeter_config(
    db_conn_pool: &PgPool,
    id: Uuid,
) -> Result<BudgeterConfig, sqlx::Error> {
    postgres::get_budgeter_config(db_conn_pool, id).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_budgeter_config_by_name(
    db_conn_pool: &PgPool,
    name: &str,
) -> Result<BudgeterConfig, sqlx::Error> {
    postgres::get_budgeter_config_by_name(db_conn_pool, name).await
}

#[tracing::instrument(skip_all)]
pub async fn update_budgeter_config(
    db_conn_pool: &PgPool,
    budgeter_config: &BudgeterConfig,
) -> Result<(), sqlx::Error> {
    postgres::update_budgeter_config(db_conn_pool, budgeter_config).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn delete_budgeter_config(db_conn_pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    postgres::delete_budgeter_config(db_conn_pool, id).await
}
