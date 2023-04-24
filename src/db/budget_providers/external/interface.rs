use ::redis::{Connection, RedisResult};
use sqlx::PgPool;

use crate::web_scraper::account::WebScrapingAccount;

use super::postgres;
use super::redis;

#[tracing::instrument(skip_all)]
pub async fn add_new_external_account(
    db_conn_pool: &PgPool,
    account: &WebScrapingAccount,
) -> Result<(), sqlx::Error> {
    postgres::add_new_external_account(db_conn_pool, account).await
}

#[tracing::instrument(skip_all)]
pub async fn update_external_account(
    db_conn_pool: &PgPool,
    account: &WebScrapingAccount,
) -> Result<(), sqlx::Error> {
    postgres::update_external_account(db_conn_pool, account).await
}

#[tracing::instrument(skip_all)]
pub async fn get_all_external_accounts(
    db_conn_pool: &PgPool,
) -> Result<Vec<WebScrapingAccount>, sqlx::Error> {
    postgres::get_all_external_accounts(db_conn_pool).await
}

#[tracing::instrument(skip_all)]
pub async fn get_external_account_by_name(
    db_conn_pool: &PgPool,
    name: &str,
) -> Result<Option<WebScrapingAccount>, sqlx::Error> {
    postgres::get_external_account_by_name(db_conn_pool, name).await
}

pub fn get_encryption_key(redis_conn: &mut Connection) -> Option<Vec<u8>> {
    redis::get_encryption_key(redis_conn)
}

pub fn set_encryption_key(
    redis_conn: &mut Connection,
    encryption_key_str: &[u8],
) -> RedisResult<()> {
    redis::set_encryption_key(redis_conn, encryption_key_str)
}
