use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{external::EncryptionKeyRepo, DbResult},
};
use fred::{clients::RedisPool, interfaces::KeysInterface};

#[derive(Clone)]
pub struct RedisEncryptionKeyRepo {
    pub redis_conn_pool: RedisPool,
}

impl RedisEncryptionKeyRepo {
    pub fn new_arced(redis_conn_pool: RedisPool) -> Arc<Self> {
        Arc::new(Self { redis_conn_pool })
    }
}

#[async_trait]
impl EncryptionKeyRepo for RedisEncryptionKeyRepo {
    #[tracing::instrument(skip(self))]
    async fn get(&self) -> DbResult<Vec<u8>> {
        self.redis_conn_pool
            .get("encryption_key")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn set(&self, encryption_key_str: &[u8]) -> DbResult<()> {
        self.redis_conn_pool
            .set("encryption_key", encryption_key_str, None, None, false)
            .await?;
        Ok(())
    }
}
