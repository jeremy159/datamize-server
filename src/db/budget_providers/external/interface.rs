use async_trait::async_trait;
use uuid::Uuid;

use crate::{error::DatamizeResult, models::budget_providers::WebScrapingAccount};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ExternalAccountRepo {
    async fn get_all(&self) -> DatamizeResult<Vec<WebScrapingAccount>>;
    async fn get(&self, account_id: Uuid) -> DatamizeResult<WebScrapingAccount>;
    async fn get_by_name(&self, name: &str) -> DatamizeResult<WebScrapingAccount>;
    async fn add(&self, account: &WebScrapingAccount) -> DatamizeResult<()>;
    async fn update(&self, account: &WebScrapingAccount) -> DatamizeResult<()>;
    async fn delete(&self, account_id: Uuid) -> DatamizeResult<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EncryptionKeyRepo {
    async fn get(&mut self) -> DatamizeResult<Vec<u8>>;
    async fn set(&mut self, encryption_key_str: &[u8]) -> DatamizeResult<()>;
}
