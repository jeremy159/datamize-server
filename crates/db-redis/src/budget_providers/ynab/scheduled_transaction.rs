use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{ynab::YnabScheduledTransactionMetaRepo, DbResult},
};
use fred::{clients::RedisPool, interfaces::KeysInterface};

#[derive(Clone)]
pub struct RedisYnabScheduledTransactionMetaRepo {
    pub redis_conn_pool: RedisPool,
}

impl RedisYnabScheduledTransactionMetaRepo {
    pub fn new_arced(redis_conn_pool: RedisPool) -> Arc<Self> {
        Arc::new(Self { redis_conn_pool })
    }
}

#[async_trait]
impl YnabScheduledTransactionMetaRepo for RedisYnabScheduledTransactionMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&self) -> DbResult<i64> {
        self.redis_conn_pool
            .get("scheduled_transactions_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&self, server_knowledge: i64) -> DbResult<()> {
        self.redis_conn_pool
            .set(
                "scheduled_transactions_delta",
                server_knowledge,
                None,
                None,
                false,
            )
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn del_delta(&self) -> DbResult<()> {
        self.redis_conn_pool
            .del("scheduled_transactions_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get_last_saved(&self) -> DbResult<String> {
        self.redis_conn_pool
            .get("scheduled_transactions_last_saved")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_last_saved(&self, last_saved: String) -> DbResult<()> {
        self.redis_conn_pool
            .set(
                "scheduled_transactions_last_saved",
                last_saved,
                None,
                None,
                false,
            )
            .await?;
        Ok(())
    }
}
