use datamize_domain::{
    async_trait,
    db::{ynab::YnabAccountMetaRepo, DbResult},
};
use redis::{aio::ConnectionManager, AsyncCommands};

#[derive(Clone)]
pub struct RedisYnabAccountMetaRepo {
    pub redis_conn: ConnectionManager,
}

impl RedisYnabAccountMetaRepo {
    pub fn new_boxed(redis_conn: ConnectionManager) -> Box<Self> {
        Box::new(Self { redis_conn })
    }
}

#[async_trait]
impl YnabAccountMetaRepo for RedisYnabAccountMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&mut self) -> DbResult<i64> {
        self.redis_conn
            .get("accounts_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DbResult<()> {
        self.redis_conn
            .set("accounts_delta", server_knowledge)
            .await?;
        Ok(())
    }
}
