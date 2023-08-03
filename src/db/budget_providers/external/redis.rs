use async_trait::async_trait;
use r2d2::PooledConnection;
use redis::{Client, Commands};

use crate::error::DatamizeResult;

use super::EncryptionKeyRepo;

pub struct RedisEncryptionKeyRepo {
    pub redis_conn: PooledConnection<Client>,
}

#[async_trait]
impl EncryptionKeyRepo for RedisEncryptionKeyRepo {
    #[tracing::instrument(skip(self))]
    async fn get(&mut self) -> DatamizeResult<Vec<u8>> {
        self.redis_conn.get("encryption_key").map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn set(&mut self, encryption_key_str: &[u8]) -> DatamizeResult<()> {
        self.redis_conn.set("encryption_key", encryption_key_str)?;
        Ok(())
    }
}
