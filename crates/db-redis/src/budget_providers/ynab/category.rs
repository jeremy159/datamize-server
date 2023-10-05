use datamize_domain::{
    async_trait,
    db::{ynab::YnabCategoryMetaRepo, DbResult},
};
use redis::{aio::ConnectionManager, AsyncCommands};

#[derive(Clone)]
pub struct RedisYnabCategoryMetaRepo {
    pub redis_conn: ConnectionManager,
}

impl RedisYnabCategoryMetaRepo {
    pub fn new_boxed(redis_conn: ConnectionManager) -> Box<Self> {
        Box::new(Self { redis_conn })
    }
}

#[async_trait]
impl YnabCategoryMetaRepo for RedisYnabCategoryMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&mut self) -> DbResult<i64> {
        self.redis_conn
            .get("categories_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DbResult<()> {
        self.redis_conn
            .set("categories_delta", server_knowledge)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn del_delta(&mut self) -> DbResult<i64> {
        self.redis_conn
            .get_del("categories_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get_last_saved(&mut self) -> DbResult<String> {
        self.redis_conn
            .get("categories_last_saved")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_last_saved(&mut self, last_saved: String) -> DbResult<()> {
        self.redis_conn
            .set("categories_last_saved", last_saved)
            .await?;
        Ok(())
    }
}