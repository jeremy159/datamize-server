use crate::config::{DatabaseSettings, RedisSettings};
use anyhow::{Context, Ok, Result};
pub use redis::Connection as RedisConnection;
pub use secrecy;
pub use sqlx::{error as sqlx_error, postgres::PgPoolOptions, PgPool};

pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod routes;
pub mod services;
pub mod startup;
pub mod telemetry;
pub mod web_scraper;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn get_redis_connection_pool(
    configuration: &RedisSettings,
) -> Result<r2d2::Pool<redis::Client>> {
    let redis_client = redis::Client::open(configuration.connection_string())
        .context("failed to establish connection to the redis instance")?;
    Ok(r2d2::Pool::new(redis_client).context("failed to create pool of redis connections")?)
}

pub fn get_redis_conn(
    pool: &r2d2::Pool<redis::Client>,
) -> Result<r2d2::PooledConnection<redis::Client>, r2d2::Error> {
    pool.get()
}
