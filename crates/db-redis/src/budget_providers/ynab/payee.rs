use datamize_domain::{
    async_trait,
    db::{ynab::YnabPayeeMetaRepo, DbResult},
};
use redis::{aio::ConnectionManager, AsyncCommands};

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
    async fn get_delta(&mut self) -> DbResult<i64> {
        self.redis_conn
            .get("payees_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DbResult<()> {
        self.redis_conn
            .set("payees_delta", server_knowledge)
            .await?;
        Ok(())
    }
}
