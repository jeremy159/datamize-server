use ::redis::{Connection, RedisResult};
use sqlx::PgPool;
use ynab::types::Account;
use ynab::types::Category;
use ynab::types::ScheduledTransactionDetail;

use super::postgres;
use super::redis;

pub async fn save_categories(
    db_conn_pool: &PgPool,
    categories: &[Category],
) -> Result<(), sqlx::Error> {
    postgres::save_categories(db_conn_pool, categories).await
}

pub async fn get_categories(db_conn_pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
    postgres::get_categories(db_conn_pool).await
}

pub async fn get_category_by_id(
    db_conn_pool: &PgPool,
    cat_id: &uuid::Uuid,
) -> Result<Option<Category>, sqlx::Error> {
    postgres::get_category_by_id(db_conn_pool, cat_id).await
}

pub async fn save_scheduled_transactions(
    db_conn_pool: &PgPool,
    scheduled_transactions: &[ScheduledTransactionDetail],
) -> Result<(), sqlx::Error> {
    postgres::save_scheduled_transactions(db_conn_pool, scheduled_transactions).await
}

pub async fn get_scheduled_transactions(
    db_conn_pool: &PgPool,
) -> Result<Vec<ScheduledTransactionDetail>, sqlx::Error> {
    postgres::get_scheduled_transactions(db_conn_pool).await
}

pub async fn save_accounts(db_conn_pool: &PgPool, accounts: &[Account]) -> Result<(), sqlx::Error> {
    postgres::save_accounts(db_conn_pool, accounts).await
}

pub async fn get_accounts(db_conn_pool: &PgPool) -> Result<Vec<Account>, sqlx::Error> {
    postgres::get_accounts(db_conn_pool).await
}

pub fn get_categories_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis::get_categories_delta(redis_conn)
}

pub fn set_categories_detla(redis_conn: &mut Connection, server_knowledge: i64) -> RedisResult<()> {
    redis::set_categories_detla(redis_conn, server_knowledge)
}

pub fn get_scheduled_transactions_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis::get_scheduled_transactions_delta(redis_conn)
}

pub fn set_scheduled_transactions_delta(
    redis_conn: &mut Connection,
    server_knowledge: i64,
) -> RedisResult<()> {
    redis::set_scheduled_transactions_delta(redis_conn, server_knowledge)
}

pub fn get_accounts_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis::get_accounts_delta(redis_conn)
}

pub fn set_accounts_detla(redis_conn: &mut Connection, server_knowledge: i64) -> RedisResult<()> {
    redis::set_accounts_detla(redis_conn, server_knowledge)
}
