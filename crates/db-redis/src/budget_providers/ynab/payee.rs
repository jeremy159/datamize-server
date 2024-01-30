use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{ynab::YnabPayeeMetaRepo, DbResult},
};
use fred::{clients::RedisPool, interfaces::KeysInterface};

#[derive(Clone)]
pub struct RedisYnabPayeeMetaRepo {
    pub redis_conn_pool: RedisPool,
}

impl RedisYnabPayeeMetaRepo {
    pub fn new_arced(redis_conn_pool: RedisPool) -> Arc<Self> {
        Arc::new(Self { redis_conn_pool })
    }
}

#[async_trait]
impl YnabPayeeMetaRepo for RedisYnabPayeeMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&self) -> DbResult<i64> {
        self.redis_conn_pool
            .get("payees_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&self, server_knowledge: i64) -> DbResult<()> {
        self.redis_conn_pool
            .set("payees_delta", server_knowledge, None, None, false)
            .await?;
        Ok(())
    }
}
