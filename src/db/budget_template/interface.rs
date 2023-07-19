use sqlx::PgPool;
use uuid::Uuid;

use crate::models::budget_template::{BudgeterConfig, ExpenseCategorization, ExternalExpense};

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

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_all_external_expenses(
    db_conn_pool: &PgPool,
) -> Result<Vec<ExternalExpense>, sqlx::Error> {
    postgres::get_all_external_expenses(db_conn_pool).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_external_expense(
    db_conn_pool: &PgPool,
    id: Uuid,
) -> Result<ExternalExpense, sqlx::Error> {
    postgres::get_external_expense(db_conn_pool, id).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_external_expense_by_name(
    db_conn_pool: &PgPool,
    name: &str,
) -> Result<ExternalExpense, sqlx::Error> {
    postgres::get_external_expense_by_name(db_conn_pool, name).await
}

#[tracing::instrument(skip_all)]
pub async fn update_external_expense(
    db_conn_pool: &PgPool,
    external_expense: &ExternalExpense,
) -> Result<(), sqlx::Error> {
    postgres::update_external_expense(db_conn_pool, external_expense).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn delete_external_expense(db_conn_pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    postgres::delete_external_expense(db_conn_pool, id).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_all_expenses_categorization(
    db_conn_pool: &PgPool,
) -> Result<Vec<ExpenseCategorization>, sqlx::Error> {
    postgres::get_all_expenses_categorization(db_conn_pool).await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_expense_categorization(
    db_conn_pool: &PgPool,
    id: Uuid,
) -> Result<ExpenseCategorization, sqlx::Error> {
    postgres::get_expense_categorization(db_conn_pool, id).await
}

#[tracing::instrument(skip_all)]
pub async fn update_all_expenses_categorization(
    db_conn_pool: &PgPool,
    expenses_categorization: &[ExpenseCategorization],
) -> Result<(), sqlx::Error> {
    postgres::update_all_expenses_categorization(db_conn_pool, expenses_categorization).await
}

#[tracing::instrument(skip_all)]
pub async fn update_expense_categorization(
    db_conn_pool: &PgPool,
    expense_categorization: &ExpenseCategorization,
) -> Result<(), sqlx::Error> {
    postgres::update_expense_categorization(db_conn_pool, expense_categorization).await
}
