use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};

use crate::{db::budget_providers::ynab::YnabTransactionMetaRepo, error::DatamizeResult};

#[derive(Clone)]
pub struct RedisYnabTransactionMetaRepo {
    pub redis_conn: ConnectionManager,
}

#[async_trait]
impl YnabTransactionMetaRepo for RedisYnabTransactionMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&mut self) -> DatamizeResult<i64> {
        self.redis_conn
            .get("transactions_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()> {
        self.redis_conn
            .set("transactions_delta", server_knowledge)
            .await?;
        Ok(())
    }
}
