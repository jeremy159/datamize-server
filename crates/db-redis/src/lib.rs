pub mod budget_providers;
pub mod towser_sessions_store;
use std::env;

use fred::prelude::*;
pub use fred::{
    clients::RedisPool,
    error::{RedisError, RedisErrorKind},
    interfaces::KeysInterface,
    types::{FromRedis, FromRedisKey, RedisKey, RedisValue},
};

pub async fn get_connection_pool(connection_url: &str) -> Result<RedisPool, RedisError> {
    let pool_size = env::var("REDIS_POOL_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(8);

    let config =
        RedisConfig::from_url(connection_url).expect("Failed to create redis config from url");

    let pool = Builder::from_config(config)
        .with_performance_config(|config| {
            config.auto_pipeline = true;
        })
        // use exponential backoff, starting at 100 ms and doubling on each failed attempt up to 30 sec
        .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
        .build_pool(pool_size)
        .expect("Failed to create redis pool");

    pool.init().await.expect("Failed to connect to redis");
    tracing::info!("Connected to Redis");

    Ok(pool)
}

#[cfg(any(feature = "testutils", test))]
pub async fn get_test_pool() -> RedisPool {
    use std::sync::Arc;

    use fred::mocks::SimpleMap;

    let config = RedisConfig {
        mocks: Some(Arc::new(SimpleMap::new())),
        ..Default::default()
    };
    let pool = Builder::from_config(config).build_pool(1).unwrap();
    pool.init().await.expect("Failed to connect to redis");

    pool
}
