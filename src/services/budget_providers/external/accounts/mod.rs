mod internal;

use futures::{future::BoxFuture, stream::FuturesOrdered, StreamExt};
use internal::*;

use async_trait::async_trait;
use orion::kex::SecretKey;

use crate::{
    config,
    db::budget_providers::external::{EncryptionKeyRepo, ExternalAccountRepo},
    error::DatamizeResult,
    models::budget_providers::{AccountType, ExternalAccount, WebScrapingAccount},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExternalAccountServiceExt {
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

pub struct ExternalAccountService<EAR: ExternalAccountRepo, EKR: EncryptionKeyRepo> {
    pub external_account_repo: EAR,
    pub encryption_key_repo: EKR,
}

#[async_trait]
impl<EAR, EKR> ExternalAccountServiceExt for ExternalAccountService<EAR, EKR>
where
    EAR: ExternalAccountRepo + Sync + Send,
    EKR: EncryptionKeyRepo + Sync + Send,
{
    async fn get_all_external_accounts(&self) -> DatamizeResult<Vec<ExternalAccount>> {
        Ok(self
            .external_account_repo
            .get_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

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

        Ok(updated_accounts
            .into_iter()
            .zip(initial_accounts)
            .map(
                |(updated_account_res, i_account)| match updated_account_res {
                    Ok(u_account) => {
                        if u_account.balance != i_account.balance {
                            u_account
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
                },
            )
            .collect::<Vec<_>>())
    }

    async fn create_external_account(&self, account: &WebScrapingAccount) -> DatamizeResult<()> {
        self.external_account_repo.add(account).await
    }

    async fn get_external_account_by_name(&self, name: &str) -> DatamizeResult<WebScrapingAccount> {
        self.external_account_repo.get_by_name(name).await
    }

    async fn update_external_account(&self, account: &WebScrapingAccount) -> DatamizeResult<()> {
        self.external_account_repo.update(account).await
    }

    async fn get_encryption_key(&mut self) -> DatamizeResult<Vec<u8>> {
        self.encryption_key_repo.get().await
    }

    async fn set_encryption_key(&mut self, key: &[u8]) -> DatamizeResult<()> {
        self.encryption_key_repo.set(key).await
    }
}
