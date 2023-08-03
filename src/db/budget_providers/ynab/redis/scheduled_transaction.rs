use async_trait::async_trait;
use r2d2::PooledConnection;
use redis::{Client, Commands};

use crate::{db::budget_providers::ynab::YnabScheduledTransactionMetaRepo, error::DatamizeResult};

pub struct RedisYnabScheduledTransactionMetaRepo {
    pub redis_conn: PooledConnection<Client>,
}

#[async_trait]
impl YnabScheduledTransactionMetaRepo for RedisYnabScheduledTransactionMetaRepo {
    #[tracing::instrument(skip(self))]
    async fn get_delta(&mut self) -> DatamizeResult<i64> {
        self.redis_conn
            .get("scheduled_transactions_delta")
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()> {
        self.redis_conn
            .set("scheduled_transactions_delta", server_knowledge)?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn del_delta(&mut self) -> DatamizeResult<i64> {
        self.redis_conn
            .get_del("scheduled_transactions_delta")
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get_last_saved(&mut self) -> DatamizeResult<String> {
        self.redis_conn
            .get("scheduled_transactions_last_saved")
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn set_last_saved(&mut self, last_saved: String) -> DatamizeResult<()> {
        self.redis_conn
            .set("scheduled_transactions_last_saved", last_saved)?;
        Ok(())
    }
}
