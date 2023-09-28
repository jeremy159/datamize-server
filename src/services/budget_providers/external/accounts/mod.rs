mod internal;

use dyn_clone::{clone_trait_object, DynClone};
use futures::{future::BoxFuture, stream::FuturesOrdered, StreamExt};
use internal::*;

use async_trait::async_trait;
use orion::kex::SecretKey;

use crate::{
    config,
    db::budget_providers::external::{DynEncryptionKeyRepo, DynExternalAccountRepo},
    error::DatamizeResult,
    models::budget_providers::{AccountType, ExternalAccount, WebScrapingAccount},
};

#[async_trait]
pub trait ExternalAccountServiceExt: DynClone + Send + Sync {
    async fn get_all_external_accounts(&self) -> DatamizeResult<Vec<ExternalAccount>>;
    async fn refresh_all_web_scraping_accounts(
        &mut self,
    ) -> DatamizeResult<Vec<WebScrapingAccount>>;

    async fn create_external_account(&self, account: &WebScrapingAccount) -> DatamizeResult<()>;
    async fn get_external_account_by_name(&self, name: &str) -> DatamizeResult<WebScrapingAccount>;
    async fn update_external_account(&self, account: &WebScrapingAccount) -> DatamizeResult<()>;

    async fn get_encryption_key(&mut self) -> DatamizeResult<Vec<u8>>;
    async fn set_encryption_key(&mut self, key: &[u8]) -> DatamizeResult<()>;
}

clone_trait_object!(ExternalAccountServiceExt);

pub type DynExternalAccountService = Box<dyn ExternalAccountServiceExt>;

#[cfg(test)]
mockall::mock! {
    pub ExternalAccountService {}

    impl Clone for ExternalAccountService {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl ExternalAccountServiceExt for ExternalAccountService {
        async fn get_all_external_accounts(&self) -> DatamizeResult<Vec<ExternalAccount>>;
        async fn refresh_all_web_scraping_accounts(
            &mut self,
        ) -> DatamizeResult<Vec<WebScrapingAccount>>;

        async fn create_external_account(&self, account: &WebScrapingAccount) -> DatamizeResult<()>;
        async fn get_external_account_by_name(&self, name: &str) -> DatamizeResult<WebScrapingAccount>;
        async fn update_external_account(&self, account: &WebScrapingAccount) -> DatamizeResult<()>;

        async fn get_encryption_key(&mut self) -> DatamizeResult<Vec<u8>>;
        async fn set_encryption_key(&mut self, key: &[u8]) -> DatamizeResult<()>;
    }
}

#[derive(Clone)]
pub struct ExternalAccountService {
    pub external_account_repo: DynExternalAccountRepo,
    pub encryption_key_repo: DynEncryptionKeyRepo,
}

#[async_trait]
impl ExternalAccountServiceExt for ExternalAccountService {
    #[tracing::instrument(skip(self))]
    async fn get_all_external_accounts(&self) -> DatamizeResult<Vec<ExternalAccount>> {
        Ok(self
            .external_account_repo
            .get_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn refresh_all_web_scraping_accounts(
        &mut self,
    ) -> DatamizeResult<Vec<WebScrapingAccount>> {
        let configuration = config::Settings::build()?;
        let webdriver_location = configuration.webdriver.connection_string();

        let encryption_key = match self.encryption_key_repo.get().await {
            Ok(ref val) => SecretKey::from_slice(val).unwrap_or_default(),
            Err(_) => {
                let key = SecretKey::default();
                self.encryption_key_repo
                    .set(key.unprotected_as_bytes())
                    .await?;
                key
            }
        };

        let initial_accounts = self.external_account_repo.get_all().await?;
        let updated_accounts = initial_accounts
            .clone()
            .into_iter()
            .map(|account| {
                let r: BoxFuture<_> = match account.account_type {
                    AccountType::Tfsa => Box::pin(tfsa::get_tfsa(
                        account,
                        &encryption_key,
                        &webdriver_location,
                    )),
                    AccountType::Rpp => Box::pin(rpp::get_rpp_canada_life_sandryne(
                        account,
                        &encryption_key,
                        &webdriver_location,
                    )),
                    AccountType::Rrsp => Box::pin(rrsp::get_rrsp_ia_sandryne(
                        account,
                        &encryption_key,
                        &webdriver_location,
                    )),
                    _ => Box::pin(async { Ok(account) }),
                };
                r
            })
            .collect::<FuturesOrdered<BoxFuture<_>>>()
            .collect::<Vec<_>>()
            .await;

        let mut accounts = vec![];

        for (updated_account_res, i_account) in updated_accounts.into_iter().zip(initial_accounts) {
            let account = match updated_account_res {
                Ok(u_account) => {
                    if u_account.balance != i_account.balance {
                        match self.external_account_repo.update(&u_account).await {
                            Ok(_) => u_account,
                            Err(e) => {
                                tracing::error!(
                                    error.cause_chain = ?e,
                                    error.message = %e,
                                    "Failed to save latest balance for account {}. Skipping.",
                                    i_account.name
                                );
                                i_account
                            }
                        }
                    } else {
                        i_account
                    }
                }
                Err(e) => {
                    tracing::error!(
                        error.cause_chain = ?e,
                        error.message = %e,
                        "Failed to get latest balance for account {}. Skipping.",
                        i_account.name
                    );
                    i_account
                }
            };

            accounts.push(account);
        }

        Ok(accounts)
    }

    #[tracing::instrument(skip_all)]
    async fn create_external_account(&self, account: &WebScrapingAccount) -> DatamizeResult<()> {
        self.external_account_repo.add(account).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_external_account_by_name(&self, name: &str) -> DatamizeResult<WebScrapingAccount> {
        self.external_account_repo.get_by_name(name).await
    }

    #[tracing::instrument(skip_all)]
    async fn update_external_account(&self, account: &WebScrapingAccount) -> DatamizeResult<()> {
        self.external_account_repo.update(account).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_encryption_key(&mut self) -> DatamizeResult<Vec<u8>> {
        self.encryption_key_repo.get().await
    }

    #[tracing::instrument(skip_all)]
    async fn set_encryption_key(&mut self, key: &[u8]) -> DatamizeResult<()> {
        self.encryption_key_repo.set(key).await
    }
}

impl ExternalAccountService {
    pub fn new_boxed(
        external_account_repo: DynExternalAccountRepo,
        encryption_key_repo: DynEncryptionKeyRepo,
    ) -> Box<Self> {
        Box::new(Self {
            external_account_repo,
            encryption_key_repo,
        })
    }
}
