use datamize_domain::{
    async_trait,
    db::{ynab::YnabTransactionMetaRepo, DbResult},
};
use redis::{aio::ConnectionManager, AsyncCommands};

#[derive(Clone)]
pub struct RedisYnabTransactionMetaRepo {
    pub redis_conn: ConnectionManager,
}

impl RedisYnabTransactionMetaRepo {
    pub fn new_boxed(redis_conn: ConnectionManager) -> Box<dyn YnabTransactionMetaRepo> {
        Box::new(Self { redis_conn })
    }
}

#[async_trait]
impl YnabTransactionMetaRepo for RedisYnabTransactionMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&mut self) -> DbResult<i64> {
        self.redis_conn
            .get("transactions_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DbResult<()> {
        self.redis_conn
            .set("transactions_delta", server_knowledge)
            .await?;
        Ok(())
    }
}
