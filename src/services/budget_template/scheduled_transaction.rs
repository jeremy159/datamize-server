use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use chrono::{Datelike, Local, NaiveDate};
use dyn_clone::{clone_trait_object, DynClone};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use ynab::ScheduledTransactionRequests;

use crate::{
    db::budget_providers::ynab::{
        DynYnabScheduledTransactionMetaRepo, DynYnabScheduledTransactionRepo,
        PostgresYnabScheduledTransactionRepo, RedisYnabScheduledTransactionMetaRepo,
    },
    error::DatamizeResult,
    models::budget_template::DatamizeScheduledTransaction,
};

#[async_trait]
pub trait ScheduledTransactionServiceExt: DynClone {
    async fn get_latest_scheduled_transactions(
        &mut self,
    ) -> DatamizeResult<Vec<DatamizeScheduledTransaction>>;
}

clone_trait_object!(ScheduledTransactionServiceExt);

pub type DynScheduledTransactionService = Box<dyn ScheduledTransactionServiceExt + Send + Sync>;

#[derive(Clone)]
pub struct ScheduledTransactionService {
    pub ynab_scheduled_transaction_repo: DynYnabScheduledTransactionRepo,
    pub ynab_scheduled_transaction_meta_repo: DynYnabScheduledTransactionMetaRepo,
    pub ynab_client: Arc<dyn ScheduledTransactionRequests + Send + Sync>,
}

#[async_trait]
impl ScheduledTransactionServiceExt for ScheduledTransactionService {
    #[tracing::instrument(skip(self))]
    async fn get_latest_scheduled_transactions(
        &mut self,
    ) -> DatamizeResult<Vec<DatamizeScheduledTransaction>> {
        self.check_last_saved().await?;
        let saved_scheduled_transactions_delta = self
            .ynab_scheduled_transaction_meta_repo
            .get_delta()
            .await
            .ok();

        let scheduled_transactions_delta = self
            .ynab_client
            .get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
            .await
            .context("failed to get scheduled transactions from ynab's API")?;

        self.ynab_scheduled_transaction_repo
            .update_all(&scheduled_transactions_delta.scheduled_transactions)
            .await
            .context("failed to save scheduled transactions in database")?;

        self.ynab_scheduled_transaction_meta_repo
            .set_delta(scheduled_transactions_delta.server_knowledge)
            .await
            .context(
                "failed to save last known server knowledge of scheduled transactions in redis",
            )?;

        Ok(self
            .ynab_scheduled_transaction_repo
            .get_all()
            .await
            .context("failed to get scheduled transactions from database")?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}

impl ScheduledTransactionService {
    pub fn new_boxed(
        db_conn_pool: PgPool,
        redis_conn: ConnectionManager,
        ynab_client: Arc<dyn ScheduledTransactionRequests + Send + Sync>,
    ) -> Box<Self> {
        Box::new(ScheduledTransactionService {
            ynab_scheduled_transaction_repo: Box::new(PostgresYnabScheduledTransactionRepo {
                db_conn_pool,
            }),
            ynab_scheduled_transaction_meta_repo: Box::new(RedisYnabScheduledTransactionMetaRepo {
                redis_conn,
            }),
            ynab_client,
        })
    }

    async fn check_last_saved(&mut self) -> DatamizeResult<()> {
        let current_date = Local::now().date_naive();
        if let Ok(last_saved) = self
            .ynab_scheduled_transaction_meta_repo
            .get_last_saved()
            .await
        {
            let last_saved_date: NaiveDate = last_saved.parse()?;
            if current_date.month() != last_saved_date.month() {
                tracing::debug!(
                    ?current_date,
                    ?last_saved_date,
                    "discarding knowledge_server",
                );
                // Discard knowledge_server when changing month.
                self.ynab_scheduled_transaction_meta_repo
                    .del_delta()
                    .await?;
                self.ynab_scheduled_transaction_meta_repo
                    .set_last_saved(current_date.to_string())
                    .await?;
            }
        } else {
            self.ynab_scheduled_transaction_meta_repo
                .set_last_saved(current_date.to_string())
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Months;
    use fake::{Fake, Faker};
    use mockall::{mock, predicate::eq};
    use ynab::{ScheduledTransactionDetail, ScheduledTransactionsDetailDelta, YnabResult};

    use super::*;
    use crate::{
        db::budget_providers::ynab::{
            MockYnabScheduledTransactionMetaRepoImpl, MockYnabScheduledTransactionRepoImpl,
        },
        error::AppError,
    };

    mock! {
        YnabClient {}
        #[async_trait]
        impl ScheduledTransactionRequests for YnabClient {
            async fn get_scheduled_transactions_delta(&self, knowledge_server: Option<i64>) -> YnabResult<ScheduledTransactionsDetailDelta>;

            async fn get_scheduled_transactions(&self) -> YnabResult<Vec<ScheduledTransactionDetail>>;

            async fn get_scheduled_transaction_by_id(
                &self,
                scheduled_transaction_id: &str,
            ) -> YnabResult<ScheduledTransactionDetail>;
        }
    }

    #[tokio::test]
    async fn check_last_saved_when_nothing_currently_saved_should_update_last_saved() {
        let ynab_scheduled_transaction_repo = Box::new(MockYnabScheduledTransactionRepoImpl::new());
        let mut ynab_scheduled_transaction_meta_repo =
            Box::new(MockYnabScheduledTransactionMetaRepoImpl::new());

        ynab_scheduled_transaction_meta_repo
            .expect_get_last_saved()
            .once()
            .returning(|| Err(AppError::ResourceNotFound));

        let expected = Local::now().date_naive();
        ynab_scheduled_transaction_meta_repo
            .expect_set_last_saved()
            .once()
            .with(eq(expected.to_string()))
            .returning(|_| Ok(()));

        let ynab_client = MockYnabClient::new();

        let mut scheduled_transaction_service = ScheduledTransactionService {
            ynab_scheduled_transaction_repo,
            ynab_scheduled_transaction_meta_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let actual = scheduled_transaction_service.check_last_saved().await;
        assert!(matches!(actual, Ok(())));
    }

    #[tokio::test]
    async fn check_last_saved_when_saved_date_is_the_same_month_as_current_should_not_update_last_saved(
    ) {
        let ynab_scheduled_transaction_repo = Box::new(MockYnabScheduledTransactionRepoImpl::new());
        let mut ynab_scheduled_transaction_meta_repo =
            Box::new(MockYnabScheduledTransactionMetaRepoImpl::new());

        let saved_date = Local::now().date_naive();
        ynab_scheduled_transaction_meta_repo
            .expect_get_last_saved()
            .once()
            .returning(move || Ok(saved_date.to_string()));

        ynab_scheduled_transaction_meta_repo
            .expect_set_last_saved()
            .never();

        let ynab_client = MockYnabClient::new();

        let mut scheduled_transaction_service = ScheduledTransactionService {
            ynab_scheduled_transaction_repo,
            ynab_scheduled_transaction_meta_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let actual = scheduled_transaction_service.check_last_saved().await;
        assert!(matches!(actual, Ok(())));
    }

    #[tokio::test]
    async fn check_last_saved_when_saved_date_is_not_the_same_month_as_current_should_update_last_saved_and_delete_delta(
    ) {
        let ynab_scheduled_transaction_repo = Box::new(MockYnabScheduledTransactionRepoImpl::new());
        let mut ynab_scheduled_transaction_meta_repo =
            Box::new(MockYnabScheduledTransactionMetaRepoImpl::new());

        let saved_date = Local::now()
            .date_naive()
            .checked_sub_months(Months::new(1))
            .unwrap();
        ynab_scheduled_transaction_meta_repo
            .expect_get_last_saved()
            .once()
            .returning(move || Ok(saved_date.to_string()));

        let expected = Local::now().date_naive();
        ynab_scheduled_transaction_meta_repo
            .expect_set_last_saved()
            .once()
            .with(eq(expected.to_string()))
            .returning(|_| Ok(()));

        ynab_scheduled_transaction_meta_repo
            .expect_del_delta()
            .once()
            .returning(|| Ok(Faker.fake()));

        let ynab_client = MockYnabClient::new();

        let mut scheduled_transaction_service = ScheduledTransactionService {
            ynab_scheduled_transaction_repo,
            ynab_scheduled_transaction_meta_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let actual = scheduled_transaction_service.check_last_saved().await;
        assert!(matches!(actual, Ok(())));
    }

    #[tokio::test]
    async fn get_latest_scheduled_transactions_should_return_all_scheduled_transactions() {
        let mut ynab_scheduled_transaction_repo =
            Box::new(MockYnabScheduledTransactionRepoImpl::new());
        let mut ynab_scheduled_transaction_meta_repo =
            Box::new(MockYnabScheduledTransactionMetaRepoImpl::new());
        let mut ynab_client = MockYnabClient::new();

        let saved_date = Local::now().date_naive();
        ynab_scheduled_transaction_meta_repo
            .expect_get_last_saved()
            .once()
            .returning(move || Ok(saved_date.to_string()));

        ynab_scheduled_transaction_meta_repo
            .expect_get_delta()
            .once()
            .returning(move || Err(AppError::ResourceNotFound));

        let expected = ScheduledTransactionsDetailDelta {
            server_knowledge: Faker.fake(),
            scheduled_transactions: vec![
                ScheduledTransactionDetail {
                    id: Faker.fake(),
                    date_first: Faker.fake(),
                    date_next: Faker.fake(),
                    frequency: None,
                    amount: Faker.fake(),
                    memo: Faker.fake(),
                    flag_color: Faker.fake(),
                    account_id: Faker.fake(),
                    payee_id: Faker.fake(),
                    category_id: Faker.fake(),
                    transfer_account_id: Faker.fake(),
                    deleted: Faker.fake(),
                    account_name: Faker.fake(),
                    payee_name: Faker.fake(),
                    category_name: Faker.fake(),
                    subtransactions: vec![],
                },
                ScheduledTransactionDetail {
                    id: Faker.fake(),
                    date_first: Faker.fake(),
                    date_next: Faker.fake(),
                    frequency: None,
                    amount: Faker.fake(),
                    memo: Faker.fake(),
                    flag_color: Faker.fake(),
                    account_id: Faker.fake(),
                    payee_id: Faker.fake(),
                    category_id: Faker.fake(),
                    transfer_account_id: Faker.fake(),
                    deleted: Faker.fake(),
                    account_name: Faker.fake(),
                    payee_name: Faker.fake(),
                    category_name: Faker.fake(),
                    subtransactions: vec![],
                },
            ],
        };
        let expected_cloned = expected.clone();
        ynab_client
            .expect_get_scheduled_transactions_delta()
            .once()
            .returning(move |_| Ok(expected_cloned.clone()));

        let expected_transactions = expected.scheduled_transactions.clone();
        ynab_scheduled_transaction_repo
            .expect_update_all()
            .once()
            .with(eq(expected_transactions.clone()))
            .returning(|_| Ok(()));
        ynab_scheduled_transaction_meta_repo
            .expect_set_delta()
            .once()
            .with(eq(expected.server_knowledge))
            .returning(|_| Ok(()));

        let expected_transactions = expected.scheduled_transactions.clone();
        ynab_scheduled_transaction_repo
            .expect_get_all()
            .once()
            .return_once(move || Ok(expected_transactions));

        let mut scheduled_transaction_service = ScheduledTransactionService {
            ynab_scheduled_transaction_repo,
            ynab_scheduled_transaction_meta_repo,
            ynab_client: Arc::new(ynab_client),
        };

        scheduled_transaction_service
            .get_latest_scheduled_transactions()
            .await
            .unwrap();
    }
}
