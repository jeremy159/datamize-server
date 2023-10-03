use datamize_domain::{
    async_trait,
    db::{external::EncryptionKeyRepo, DbResult},
};
use redis::{aio::ConnectionManager, AsyncCommands};

#[derive(Clone)]
pub struct RedisEncryptionKeyRepo {
    pub redis_conn: ConnectionManager,
}

impl RedisEncryptionKeyRepo {
    pub fn new_boxed(redis_conn: ConnectionManager) -> Box<Self> {
        Box::new(Self { redis_conn })
    }
}

#[async_trait]
impl EncryptionKeyRepo for RedisEncryptionKeyRepo {
    #[tracing::instrument(skip(self))]
    async fn get(&mut self) -> DbResult<Vec<u8>> {
        self.redis_conn
            .get("encryption_key")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn set(&mut self, encryption_key_str: &[u8]) -> DbResult<()> {
        self.redis_conn
            .set("encryption_key", encryption_key_str)
            .await?;
        Ok(())
    }
}
