use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{ynab::YnabTransactionMetaRepo, DbResult},
};
use fred::{clients::RedisPool, interfaces::KeysInterface};

#[derive(Clone)]
pub struct RedisYnabTransactionMetaRepo {
    pub redis_conn_pool: RedisPool,
}

impl RedisYnabTransactionMetaRepo {
    pub fn new_arced(redis_conn_pool: RedisPool) -> Arc<dyn YnabTransactionMetaRepo> {
        Arc::new(Self { redis_conn_pool })
    }
}

#[async_trait]
impl YnabTransactionMetaRepo for RedisYnabTransactionMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&self) -> DbResult<i64> {
        self.redis_conn_pool
            .get("transactions_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&self, server_knowledge: i64) -> DbResult<()> {
        self.redis_conn_pool
            .set("transactions_delta", server_knowledge, None, None, false)
            .await?;
        Ok(())
    }
}
