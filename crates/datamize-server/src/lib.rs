use crate::config::{DatabaseSettings, RedisSettings};
use anyhow::Context;
use db_redis::redis::aio::ConnectionManager;
pub use db_redis::redis::Connection as RedisConnection;
pub use sqlx::{error as sqlx_error, postgres::PgPoolOptions, PgPool};

pub mod config;
pub mod error;
pub mod routes;
pub mod services;
pub mod startup;
pub mod telemetry;

use error::DatamizeResult;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub async fn get_redis_connection_manager(
    configuration: &RedisSettings,
) -> DatamizeResult<ConnectionManager> {
    let client = db_redis::redis::Client::open(configuration.connection_string())
        .context("failed to establish connection to the redis instance")?;
    Ok(ConnectionManager::new(client)
        .await
        .context("failed to create redis connection manager")?)
}
