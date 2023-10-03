use datamize_domain::{
    async_trait,
    db::{ynab::YnabScheduledTransactionMetaRepo, DbResult},
};
use redis::{aio::ConnectionManager, AsyncCommands};

#[derive(Clone)]
pub struct RedisYnabScheduledTransactionMetaRepo {
    pub redis_conn: ConnectionManager,
}

impl RedisYnabScheduledTransactionMetaRepo {
    pub fn new_boxed(redis_conn: ConnectionManager) -> Box<Self> {
        Box::new(Self { redis_conn })
    }
}

#[async_trait]
impl YnabScheduledTransactionMetaRepo for RedisYnabScheduledTransactionMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&mut self) -> DbResult<i64> {
        self.redis_conn
            .get("scheduled_transactions_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DbResult<()> {
        self.redis_conn
            .set("scheduled_transactions_delta", server_knowledge)
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn del_delta(&mut self) -> DbResult<i64> {
        self.redis_conn
            .get_del("scheduled_transactions_delta")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get_last_saved(&mut self) -> DbResult<String> {
        self.redis_conn
            .get("scheduled_transactions_last_saved")
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_last_saved(&mut self, last_saved: String) -> DbResult<()> {
        self.redis_conn
            .set("scheduled_transactions_last_saved", last_saved)
            .await?;
        Ok(())
    }
}
