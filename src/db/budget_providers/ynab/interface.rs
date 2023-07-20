use ::redis::{Connection, RedisResult};
use sqlx::PgPool;
use ynab::types::Account;
use ynab::types::Category;
use ynab::types::Payee;
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

pub async fn save_payees(db_conn_pool: &PgPool, payees: &[Payee]) -> Result<(), sqlx::Error> {
    postgres::save_payees(db_conn_pool, payees).await
}

pub async fn get_payees(db_conn_pool: &PgPool) -> Result<Vec<Payee>, sqlx::Error> {
    postgres::get_payees(db_conn_pool).await
}

pub fn get_categories_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis::get_categories_delta(redis_conn)
}

pub fn set_categories_detla(redis_conn: &mut Connection, server_knowledge: i64) -> RedisResult<()> {
    redis::set_categories_detla(redis_conn, server_knowledge)
}

pub fn del_categories_detla(redis_conn: &mut Connection) -> RedisResult<()> {
    redis::del_categories_detla(redis_conn)
}

pub fn get_categories_last_saved(redis_conn: &mut Connection) -> Option<String> {
    redis::get_categories_last_saved(redis_conn)
}

pub fn set_categories_last_saved(
    redis_conn: &mut Connection,
    last_saved: String,
) -> RedisResult<()> {
    redis::set_categories_last_saved(redis_conn, last_saved)
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

pub fn del_scheduled_transactions_delta(redis_conn: &mut Connection) -> RedisResult<()> {
    redis::del_scheduled_transactions_delta(redis_conn)
}

pub fn get_scheduled_transactions_last_saved(redis_conn: &mut Connection) -> Option<String> {
    redis::get_scheduled_transactions_last_saved(redis_conn)
}

pub fn set_scheduled_transactions_last_saved(
    redis_conn: &mut Connection,
    last_saved: String,
) -> RedisResult<()> {
    redis::set_scheduled_transactions_last_saved(redis_conn, last_saved)
}

pub fn get_accounts_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis::get_accounts_delta(redis_conn)
}

pub fn set_accounts_detla(redis_conn: &mut Connection, server_knowledge: i64) -> RedisResult<()> {
    redis::set_accounts_detla(redis_conn, server_knowledge)
}

pub fn get_payees_delta(redis_conn: &mut Connection) -> Option<i64> {
    redis::get_payees_delta(redis_conn)
}

pub fn set_payees_detla(redis_conn: &mut Connection, server_knowledge: i64) -> RedisResult<()> {
    redis::set_payees_detla(redis_conn, server_knowledge)
}
