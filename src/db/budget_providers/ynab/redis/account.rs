use async_trait::async_trait;
use r2d2::PooledConnection;
use redis::{Client, Commands};

use crate::{db::budget_providers::ynab::YnabAccountMetaRepo, error::DatamizeResult};

pub struct RedisYnabAccountMetaRepo {
    pub redis_conn: PooledConnection<Client>,
}

#[async_trait]
impl YnabAccountMetaRepo for RedisYnabAccountMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&mut self) -> DatamizeResult<i64> {
        self.redis_conn.get("accounts_delta").map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()> {
        self.redis_conn.set("accounts_delta", server_knowledge)?;
        Ok(())
    }
}
