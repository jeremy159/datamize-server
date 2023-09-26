use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;
use ynab::{TransactionDetail, TransactionRequests};

use crate::{
    db::budget_providers::ynab::{
        DynYnabTransactionMetaRepo, DynYnabTransactionRepo, PostgresYnabTransactionRepo,
        RedisYnabTransactionMetaRepo,
    },
    error::DatamizeResult,
};

#[async_trait]
pub trait TransactionServiceExt: DynClone {
    async fn refresh_saved_transactions(&mut self) -> DatamizeResult<()>;
    async fn get_latest_transactions(&mut self) -> DatamizeResult<Vec<TransactionDetail>>;
    async fn get_transactions_by_category_id(
        &self,
        category_id: Uuid,
    ) -> DatamizeResult<Vec<TransactionDetail>>;
    async fn get_transactions_by_payee_id(
        &self,
        payee_id: Uuid,
    ) -> DatamizeResult<Vec<TransactionDetail>>;
}

clone_trait_object!(TransactionServiceExt);

pub type DynTransactionService = Box<dyn TransactionServiceExt + Send + Sync>;

#[derive(Clone)]
pub struct TransactionService {
    pub ynab_transaction_repo: DynYnabTransactionRepo,
    pub ynab_transaction_meta_repo: DynYnabTransactionMetaRepo,
    pub ynab_client: Arc<dyn TransactionRequests + Send + Sync>,
}

#[async_trait]
impl TransactionServiceExt for TransactionService {
    #[tracing::instrument(skip(self))]
    async fn refresh_saved_transactions(&mut self) -> DatamizeResult<()> {
        let saved_transactions_delta = self.ynab_transaction_meta_repo.get_delta().await.ok();

        let transactions_delta = self
            .ynab_client
            .get_transactions_delta(saved_transactions_delta)
            .await
            .context("failed to get transactions from ynab's API")?;

        self.ynab_transaction_repo
            .update_all(&transactions_delta.transactions)
            .await
            .context("failed to save transactions in database")?;

        self.ynab_transaction_meta_repo
            .set_delta(transactions_delta.server_knowledge)
            .await
            .context("failed to save last known server knowledge of transactions in redis")?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_latest_transactions(&mut self) -> DatamizeResult<Vec<TransactionDetail>> {
        self.refresh_saved_transactions().await?;

        Ok(self
            .ynab_transaction_repo
            .get_all()
            .await
            .context("failed to get transactions from database")?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get_transactions_by_category_id(
        &self,
        category_id: Uuid,
    ) -> DatamizeResult<Vec<TransactionDetail>> {
        self.ynab_transaction_repo
            .get_all_with_category_id(category_id)
            .await
    }

    #[tracing::instrument(skip(self))]
    async fn get_transactions_by_payee_id(
        &self,
        payee_id: Uuid,
    ) -> DatamizeResult<Vec<TransactionDetail>> {
        self.ynab_transaction_repo
            .get_all_with_payee_id(payee_id)
            .await
    }
}

impl TransactionService {
    pub fn new_boxed(
        db_conn_pool: PgPool,
        redis_conn: ConnectionManager,
        ynab_client: Arc<dyn TransactionRequests + Send + Sync>,
    ) -> Box<Self> {
        Box::new(TransactionService {
            ynab_transaction_repo: Box::new(PostgresYnabTransactionRepo { db_conn_pool }),
            ynab_transaction_meta_repo: Box::new(RedisYnabTransactionMetaRepo { redis_conn }),
            ynab_client,
        })
    }
}
