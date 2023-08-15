use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use uuid::Uuid;

use crate::{error::DatamizeResult, models::budget_providers::WebScrapingAccount};

#[async_trait]
pub trait ExternalAccountRepo: DynClone {
    async fn get_all(&self) -> DatamizeResult<Vec<WebScrapingAccount>>;
    async fn get(&self, account_id: Uuid) -> DatamizeResult<WebScrapingAccount>;
    async fn get_by_name(&self, name: &str) -> DatamizeResult<WebScrapingAccount>;
    async fn add(&self, account: &WebScrapingAccount) -> DatamizeResult<()>;
    async fn update(&self, account: &WebScrapingAccount) -> DatamizeResult<()>;
    async fn delete(&self, account_id: Uuid) -> DatamizeResult<()>;
}

clone_trait_object!(ExternalAccountRepo);

pub type DynExternalAccountRepo = Box<dyn ExternalAccountRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub ExternalAccountRepoImpl {}

    impl Clone for ExternalAccountRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl ExternalAccountRepo for ExternalAccountRepoImpl {
        async fn get_all(&self) -> DatamizeResult<Vec<WebScrapingAccount>>;
        async fn get(&self, account_id: Uuid) -> DatamizeResult<WebScrapingAccount>;
        async fn get_by_name(&self, name: &str) -> DatamizeResult<WebScrapingAccount>;
        async fn add(&self, account: &WebScrapingAccount) -> DatamizeResult<()>;
        async fn update(&self, account: &WebScrapingAccount) -> DatamizeResult<()>;
        async fn delete(&self, account_id: Uuid) -> DatamizeResult<()>;
    }
}

#[async_trait]
pub trait EncryptionKeyRepo: DynClone {
    async fn get(&mut self) -> DatamizeResult<Vec<u8>>;
    async fn set(&mut self, encryption_key_str: &[u8]) -> DatamizeResult<()>;
}

clone_trait_object!(EncryptionKeyRepo);

pub type DynEncryptionKeyRepo = Box<dyn EncryptionKeyRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub EncryptionKeyRepoImpl {}

    impl Clone for EncryptionKeyRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl EncryptionKeyRepo for EncryptionKeyRepoImpl {
        async fn get(&mut self) -> DatamizeResult<Vec<u8>>;
        async fn set(&mut self, encryption_key_str: &[u8]) -> DatamizeResult<()>;
    }
}
