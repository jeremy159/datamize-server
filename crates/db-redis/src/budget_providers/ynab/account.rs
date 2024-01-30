use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{ynab::YnabAccountMetaRepo, DbResult},
};
use fred::{clients::RedisPool, interfaces::KeysInterface};

#[derive(Clone)]
pub struct RedisYnabAccountMetaRepo {
    pub redis_conn_pool: RedisPool,
}

impl RedisYnabAccountMetaRepo {
    pub fn new_arced(redis_conn_pool: RedisPool) -> Arc<Self> {
        Arc::new(Self { redis_conn_pool })
    }
}

#[async_trait]
impl YnabAccountMetaRepo for RedisYnabAccountMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&self) -> DbResult<i64> {
        self.redis_conn_pool
            .get("accounts_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&self, server_knowledge: i64) -> DbResult<()> {
        self.redis_conn_pool
            .set("accounts_delta", server_knowledge, None, None, false)
            .await?;
        Ok(())
    }
}
