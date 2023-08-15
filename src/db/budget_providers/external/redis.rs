use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};

use crate::error::DatamizeResult;

use super::EncryptionKeyRepo;

#[derive(Clone)]
pub struct RedisEncryptionKeyRepo {
    pub redis_conn: ConnectionManager,
}

#[async_trait]
impl EncryptionKeyRepo for RedisEncryptionKeyRepo {
    #[tracing::instrument(skip(self))]
    async fn get(&mut self) -> DatamizeResult<Vec<u8>> {
        self.redis_conn
            .get("encryption_key")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn set(&mut self, encryption_key_str: &[u8]) -> DatamizeResult<()> {
        self.redis_conn
            .set("encryption_key", encryption_key_str)
            .await?;
        Ok(())
    }
}
