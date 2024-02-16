use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::ynab::{DynYnabAccountMetaRepo, DynYnabAccountRepo},
};
use ynab::{Account, AccountRequests};

use crate::error::DatamizeResult;

#[async_trait]
pub trait YnabAccountServiceExt: Send + Sync {
    async fn get_all_ynab_accounts(&self) -> DatamizeResult<Vec<Account>>;
}

pub type DynYnabAccountService = Arc<dyn YnabAccountServiceExt>;

#[derive(Clone)]
pub struct YnabAccountService {
    pub ynab_account_repo: DynYnabAccountRepo,
    pub ynab_account_meta_repo: DynYnabAccountMetaRepo,
    pub ynab_client: Arc<dyn AccountRequests + Send + Sync>,
}

#[async_trait]
impl YnabAccountServiceExt for YnabAccountService {
    #[tracing::instrument(skip(self))]
    async fn get_all_ynab_accounts(&self) -> DatamizeResult<Vec<Account>> {
        let saved_accounts_delta = self.ynab_account_meta_repo.get_delta().await.ok();

        let accounts_delta = self
            .ynab_client
            .get_accounts_delta(saved_accounts_delta)
            .await?;

        let accounts = accounts_delta
            .accounts
            .into_iter()
            .filter(|a| !a.deleted)
            .collect::<Vec<_>>();

        self.ynab_account_repo.update_all(&accounts).await?;

        self.ynab_account_meta_repo
            .set_delta(accounts_delta.server_knowledge)
            .await?;

        let saved_accounts = self.ynab_account_repo.get_all().await?;

        Ok(saved_accounts)
    }
}

impl YnabAccountService {
    pub fn new_arced(
        ynab_account_repo: DynYnabAccountRepo,
        ynab_account_meta_repo: DynYnabAccountMetaRepo,
        ynab_client: Arc<dyn AccountRequests + Send + Sync>,
    ) -> Arc<Self> {
        Arc::new(YnabAccountService {
            ynab_account_repo,
            ynab_account_meta_repo,
            ynab_client,
        })
    }
}
