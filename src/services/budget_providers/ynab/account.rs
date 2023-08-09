use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use ynab::{Account, AccountRequests};

use crate::{
    db::budget_providers::ynab::{YnabAccountMetaRepo, YnabAccountRepo},
    error::DatamizeResult,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabAccountServiceExt {
    async fn get_all_ynab_accounts(&mut self) -> DatamizeResult<Vec<Account>>;
}

pub struct YnabAccountService {
    pub ynab_account_repo: Box<dyn YnabAccountRepo + Sync + Send>,
    pub ynab_account_meta_repo: Box<dyn YnabAccountMetaRepo + Sync + Send>,
    pub ynab_client: Arc<dyn AccountRequests + Send + Sync>,
}

#[async_trait]
impl YnabAccountServiceExt for YnabAccountService {
    #[tracing::instrument(skip(self))]
    async fn get_all_ynab_accounts(&mut self) -> DatamizeResult<Vec<Account>> {
        let saved_accounts_delta = self.ynab_account_meta_repo.get_delta().await.ok();

        let accounts_delta = self
            .ynab_client
            .get_accounts_delta(saved_accounts_delta)
            .await
            .context("failed to get accounts from ynab's API")?;

        let accounts = accounts_delta
            .accounts
            .into_iter()
            .filter(|a| !a.deleted)
            .collect::<Vec<_>>();

        self.ynab_account_repo
            .update_all(&accounts)
            .await
            .context("failed to save accounts in database")?;

        self.ynab_account_meta_repo
            .set_delta(accounts_delta.server_knowledge)
            .await
            .context("failed to save last known server knowledge of accounts in redis")?;

        let saved_accounts = self
            .ynab_account_repo
            .get_all()
            .await
            .context("failed to get accounts from database")?;

        Ok(saved_accounts)
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use mockall::{mock, predicate::eq};
    use ynab::{AccountType, AccountsDelta, SaveAccount, YnabResult};

    use super::*;
    use crate::{
        db::budget_providers::ynab::{MockYnabAccountMetaRepo, MockYnabAccountRepo},
        error::AppError,
    };

    mock! {
        YnabClient {}
        #[async_trait]
        impl AccountRequests for YnabClient {
            async fn get_accounts(&self) -> YnabResult<Vec<Account>>;
            async fn get_accounts_delta(
                &self,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<AccountsDelta>;
            async fn create_account(&self, data: SaveAccount) -> YnabResult<Account>;
            async fn get_account_by_id(&self, account_id: &str) -> YnabResult<Account>;
        }
    }

    #[tokio::test]
    async fn get_all_ynab_accounts_success() {
        let mut ynab_account_repo = Box::new(MockYnabAccountRepo::new());
        let mut ynab_account_meta_repo = Box::new(MockYnabAccountMetaRepo::new());
        let mut ynab_client = MockYnabClient::new();

        ynab_account_meta_repo
            .expect_get_delta()
            .once()
            .returning(|| Err(AppError::ResourceNotFound));

        let accounts = vec![
            Account {
                id: Faker.fake(),
                name: Faker.fake(),
                account_type: AccountType::Cash,
                on_budget: Faker.fake(),
                closed: Faker.fake(),
                note: Faker.fake(),
                balance: Faker.fake(),
                cleared_balance: Faker.fake(),
                uncleared_balance: Faker.fake(),
                transfer_payee_id: Faker.fake(),
                direct_import_linked: Faker.fake(),
                direct_import_in_error: Faker.fake(),
                deleted: false,
            },
            Account {
                id: Faker.fake(),
                name: Faker.fake(),
                account_type: AccountType::Cash,
                on_budget: Faker.fake(),
                closed: Faker.fake(),
                note: Faker.fake(),
                balance: Faker.fake(),
                cleared_balance: Faker.fake(),
                uncleared_balance: Faker.fake(),
                transfer_payee_id: Faker.fake(),
                direct_import_linked: Faker.fake(),
                direct_import_in_error: Faker.fake(),
                deleted: false,
            },
        ];
        let accounts_cloned = accounts.clone();
        ynab_client
            .expect_get_accounts_delta()
            .once()
            .returning(move |_| {
                Ok(AccountsDelta {
                    server_knowledge: Faker.fake(),
                    accounts: accounts_cloned.clone(),
                })
            });

        ynab_account_repo
            .expect_update_all()
            .once()
            .with(eq(accounts.clone()))
            .returning(|_| Ok(()));

        ynab_account_meta_repo
            .expect_set_delta()
            .once()
            .returning(|_| Ok(()));

        ynab_account_repo
            .expect_get_all()
            .once()
            .returning(move || Ok(accounts.clone()));

        let mut ynab_account_service = YnabAccountService {
            ynab_account_repo,
            ynab_account_meta_repo,
            ynab_client: Arc::new(ynab_client),
        };

        ynab_account_service.get_all_ynab_accounts().await.unwrap();
    }

    #[tokio::test]
    async fn get_all_ynab_accounts_issue_with_db_should_not_update_saved_delta() {
        let mut ynab_account_repo = Box::new(MockYnabAccountRepo::new());
        let mut ynab_account_meta_repo = Box::new(MockYnabAccountMetaRepo::new());
        let mut ynab_client = MockYnabClient::new();

        ynab_account_meta_repo
            .expect_get_delta()
            .once()
            .returning(|| Err(AppError::ResourceNotFound));

        let accounts = vec![
            Account {
                id: Faker.fake(),
                name: Faker.fake(),
                account_type: AccountType::Cash,
                on_budget: Faker.fake(),
                closed: Faker.fake(),
                note: Faker.fake(),
                balance: Faker.fake(),
                cleared_balance: Faker.fake(),
                uncleared_balance: Faker.fake(),
                transfer_payee_id: Faker.fake(),
                direct_import_linked: Faker.fake(),
                direct_import_in_error: Faker.fake(),
                deleted: false,
            },
            Account {
                id: Faker.fake(),
                name: Faker.fake(),
                account_type: AccountType::Cash,
                on_budget: Faker.fake(),
                closed: Faker.fake(),
                note: Faker.fake(),
                balance: Faker.fake(),
                cleared_balance: Faker.fake(),
                uncleared_balance: Faker.fake(),
                transfer_payee_id: Faker.fake(),
                direct_import_linked: Faker.fake(),
                direct_import_in_error: Faker.fake(),
                deleted: false,
            },
        ];
        let accounts_cloned = accounts.clone();
        ynab_client
            .expect_get_accounts_delta()
            .once()
            .returning(move |_| {
                Ok(AccountsDelta {
                    server_knowledge: Faker.fake(),
                    accounts: accounts_cloned.clone(),
                })
            });

        ynab_account_repo.expect_update_all().once().returning(|_| {
            Err(AppError::InternalServerError(
                sqlx::Error::RowNotFound.into(),
            ))
        });

        ynab_account_meta_repo.expect_set_delta().never();

        ynab_account_repo.expect_get_all().never();

        let mut ynab_account_service = YnabAccountService {
            ynab_account_repo,
            ynab_account_meta_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let actual = ynab_account_service.get_all_ynab_accounts().await;

        assert!(matches!(actual, Err(AppError::InternalServerError(_))));
    }
}
