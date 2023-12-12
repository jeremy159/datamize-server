use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use uuid::Uuid;

use crate::{db::error::DbResult, models::WebScrapingAccount};

#[async_trait]
pub trait ExternalAccountRepo: DynClone + Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<WebScrapingAccount>>;
    async fn get(&self, account_id: Uuid) -> DbResult<WebScrapingAccount>;
    async fn get_by_name(&self, name: &str) -> DbResult<WebScrapingAccount>;
    async fn add(&self, account: &WebScrapingAccount) -> DbResult<()>;
    async fn update(&self, account: &WebScrapingAccount) -> DbResult<()>;
    async fn delete(&self, account_id: Uuid) -> DbResult<()>;
}

clone_trait_object!(ExternalAccountRepo);

pub type DynExternalAccountRepo = Box<dyn ExternalAccountRepo>;

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub ExternalAccountRepoImpl {}

    impl Clone for ExternalAccountRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl ExternalAccountRepo for ExternalAccountRepoImpl {
        async fn get_all(&self) -> DbResult<Vec<WebScrapingAccount>>;
        async fn get(&self, account_id: Uuid) -> DbResult<WebScrapingAccount>;
        async fn get_by_name(&self, name: &str) -> DbResult<WebScrapingAccount>;
        async fn add(&self, account: &WebScrapingAccount) -> DbResult<()>;
        async fn update(&self, account: &WebScrapingAccount) -> DbResult<()>;
        async fn delete(&self, account_id: Uuid) -> DbResult<()>;
    }
}

#[async_trait]
pub trait EncryptionKeyRepo: DynClone + Send + Sync {
    async fn get(&mut self) -> DbResult<Vec<u8>>;
    async fn set(&mut self, encryption_key_str: &[u8]) -> DbResult<()>;
}

clone_trait_object!(EncryptionKeyRepo);

pub type DynEncryptionKeyRepo = Box<dyn EncryptionKeyRepo>;

#[cfg(any(feature = "testutils", test))]
mod mocks {
    use super::*;
    use fake::{Fake, Faker};

    #[derive(Clone)]
    pub struct MockEncryptionKeyRepo {}

    impl MockEncryptionKeyRepo {
        pub fn new_boxed() -> Box<dyn EncryptionKeyRepo> {
            Box::new(Self {})
        }
    }

    #[async_trait]
    impl EncryptionKeyRepo for MockEncryptionKeyRepo {
        async fn get(&mut self) -> DbResult<Vec<u8>> {
            Ok(Faker.fake())
        }

        async fn set(&mut self, _encryption_key_str: &[u8]) -> DbResult<()> {
            Ok(())
        }
    }
}

#[cfg(any(feature = "testutils", test))]
pub use mocks::*;
