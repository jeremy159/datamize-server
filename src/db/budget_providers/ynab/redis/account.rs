use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};

use crate::{db::budget_providers::ynab::YnabAccountMetaRepo, error::DatamizeResult};

#[derive(Clone)]
pub struct RedisYnabAccountMetaRepo {
    pub redis_conn: ConnectionManager,
}

#[async_trait]
impl YnabAccountMetaRepo for RedisYnabAccountMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&mut self) -> DatamizeResult<i64> {
        self.redis_conn
            .get("accounts_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()> {
        self.redis_conn
            .set("accounts_delta", server_knowledge)
            .await?;
        Ok(())
    }
}
