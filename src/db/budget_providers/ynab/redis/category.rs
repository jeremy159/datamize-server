use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};

use crate::{db::budget_providers::ynab::YnabCategoryMetaRepo, error::DatamizeResult};

#[derive(Clone)]
pub struct RedisYnabCategoryMetaRepo {
    pub redis_conn: ConnectionManager,
}

#[async_trait]
impl YnabCategoryMetaRepo for RedisYnabCategoryMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&mut self) -> DatamizeResult<i64> {
        self.redis_conn
            .get("categories_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()> {
        self.redis_conn
            .set("categories_delta", server_knowledge)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn del_delta(&mut self) -> DatamizeResult<i64> {
        self.redis_conn
            .get_del("categories_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get_last_saved(&mut self) -> DatamizeResult<String> {
        self.redis_conn
            .get("categories_last_saved")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_last_saved(&mut self, last_saved: String) -> DatamizeResult<()> {
        self.redis_conn
            .set("categories_last_saved", last_saved)
            .await?;
        Ok(())
    }
}
