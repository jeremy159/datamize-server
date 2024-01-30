use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{ynab::YnabCategoryMetaRepo, DbResult},
};
use fred::{clients::RedisPool, interfaces::KeysInterface};

#[derive(Clone)]
pub struct RedisYnabCategoryMetaRepo {
    pub redis_conn_pool: RedisPool,
}

impl RedisYnabCategoryMetaRepo {
    pub fn new_arced(redis_conn_pool: RedisPool) -> Arc<Self> {
        Arc::new(Self { redis_conn_pool })
    }
}

#[async_trait]
impl YnabCategoryMetaRepo for RedisYnabCategoryMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&self) -> DbResult<i64> {
        self.redis_conn_pool
            .get("categories_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&self, server_knowledge: i64) -> DbResult<()> {
        self.redis_conn_pool
            .set("categories_delta", server_knowledge, None, None, false)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn del_delta(&self) -> DbResult<i64> {
        self.redis_conn_pool
            .getdel("categories_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get_last_saved(&self) -> DbResult<String> {
        self.redis_conn_pool
            .get("categories_last_saved")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_last_saved(&self, last_saved: String) -> DbResult<()> {
        self.redis_conn_pool
            .set("categories_last_saved", last_saved, None, None, false)
            .await?;
        Ok(())
    }
}
