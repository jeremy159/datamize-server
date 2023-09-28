use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};

use crate::{db::budget_providers::ynab::YnabPayeeMetaRepo, error::DatamizeResult};

#[derive(Clone)]
pub struct RedisYnabPayeeMetaRepo {
    pub redis_conn: ConnectionManager,
}

impl RedisYnabPayeeMetaRepo {
    pub fn new_boxed(redis_conn: ConnectionManager) -> Box<Self> {
        Box::new(Self { redis_conn })
    }
}

#[async_trait]
impl YnabPayeeMetaRepo for RedisYnabPayeeMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&mut self) -> DatamizeResult<i64> {
        self.redis_conn
            .get("payees_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()> {
        self.redis_conn
            .set("payees_delta", server_knowledge)
            .await?;
        Ok(())
    }
}
