use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{db::error::DbResult, models::WebScrapingAccount};

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait ExternalAccountRepo: Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<WebScrapingAccount>>;
    async fn get(&self, account_id: Uuid) -> DbResult<WebScrapingAccount>;
    async fn get_by_name(&self, name: &str) -> DbResult<WebScrapingAccount>;
    async fn add(&self, account: &WebScrapingAccount) -> DbResult<()>;
    async fn update(&self, account: &WebScrapingAccount) -> DbResult<()>;
    async fn delete(&self, account_id: Uuid) -> DbResult<()>;
}

pub type DynExternalAccountRepo = Arc<dyn ExternalAccountRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait EncryptionKeyRepo: Send + Sync {
    async fn get(&self) -> DbResult<Vec<u8>>;
    async fn set(&self, encryption_key_str: &[u8]) -> DbResult<()>;
}

pub type DynEncryptionKeyRepo = Arc<dyn EncryptionKeyRepo>;
