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

pub struct YnabAccountService<YAR: YnabAccountRepo, YAMR: YnabAccountMetaRepo> {
    pub ynab_account_repo: YAR,
    pub ynab_account_meta_repo: YAMR,
    pub ynab_client: Arc<dyn AccountRequests + Send + Sync>,
}

#[async_trait]
impl<YAR, YAMR> YnabAccountServiceExt for YnabAccountService<YAR, YAMR>
where
    YAR: YnabAccountRepo + Sync + Send,
    YAMR: YnabAccountMetaRepo + Sync + Send,
{
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
