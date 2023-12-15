use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use uuid::Uuid;
use ynab::types::Account;
use ynab::types::Category;
use ynab::types::Payee;
use ynab::types::ScheduledTransactionDetail;
use ynab::TransactionDetail;

use crate::db::error::DbResult;

#[async_trait]
pub trait YnabCategoryRepo: DynClone + Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<Category>>;
    async fn get(&self, category_id: Uuid) -> DbResult<Category>;
    async fn update_all(&self, categories: &[Category]) -> DbResult<()>;
}

clone_trait_object!(YnabCategoryRepo);

pub type DynYnabCategoryRepo = Box<dyn YnabCategoryRepo>;

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub YnabCategoryRepoImpl {}

    impl Clone for YnabCategoryRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabCategoryRepo for YnabCategoryRepoImpl {
        async fn get_all(&self) -> DbResult<Vec<Category>>;
        async fn get(&self, category_id: Uuid) -> DbResult<Category>;
        async fn update_all(&self, categories: &[Category]) -> DbResult<()>;
    }
}

#[async_trait]
pub trait YnabScheduledTransactionRepo: DynClone + Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<ScheduledTransactionDetail>>;
    async fn update_all(
        &self,
        scheduled_transactions: &[ScheduledTransactionDetail],
    ) -> DbResult<()>;
}

clone_trait_object!(YnabScheduledTransactionRepo);

pub type DynYnabScheduledTransactionRepo = Box<dyn YnabScheduledTransactionRepo>;

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub YnabScheduledTransactionRepoImpl {}

    impl Clone for YnabScheduledTransactionRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabScheduledTransactionRepo for YnabScheduledTransactionRepoImpl {
        async fn get_all(&self) -> DbResult<Vec<ScheduledTransactionDetail>>;
        async fn update_all(&self, scheduled_transactions: &[ScheduledTransactionDetail]) -> DbResult<()>;
    }
}

#[async_trait]
pub trait YnabAccountRepo: DynClone + Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<Account>>;
    async fn update_all(&self, accounts: &[Account]) -> DbResult<()>;
}

clone_trait_object!(YnabAccountRepo);

pub type DynYnabAccountRepo = Box<dyn YnabAccountRepo>;

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub YnabAccountRepoImpl {}

    impl Clone for YnabAccountRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabAccountRepo for YnabAccountRepoImpl {
        async fn get_all(&self) -> DbResult<Vec<Account>>;
        async fn update_all(&self, accounts: &[Account]) -> DbResult<()>;
    }
}

#[async_trait]
pub trait YnabPayeeRepo: DynClone + Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<Payee>>;
    async fn update_all(&self, payees: &[Payee]) -> DbResult<()>;
}

clone_trait_object!(YnabPayeeRepo);

pub type DynYnabPayeeRepo = Box<dyn YnabPayeeRepo>;

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub YnabPayeeRepoImpl {}

    impl Clone for YnabPayeeRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabPayeeRepo for YnabPayeeRepoImpl {
        async fn get_all(&self) -> DbResult<Vec<Payee>>;
        async fn update_all(&self, payees: &[Payee]) -> DbResult<()>;
    }
}

#[async_trait]
pub trait YnabTransactionRepo: DynClone + Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<TransactionDetail>>;
    async fn update_all(&self, transactions: &[TransactionDetail]) -> DbResult<()>;
    async fn get_all_with_payee_id(&self, payee_id: Uuid) -> DbResult<Vec<TransactionDetail>>;
    async fn get_all_with_category_id(&self, category_id: Uuid)
        -> DbResult<Vec<TransactionDetail>>;
}

clone_trait_object!(YnabTransactionRepo);

pub type DynYnabTransactionRepo = Box<dyn YnabTransactionRepo>;

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub YnabTransactionRepoImpl {}

    impl Clone for YnabTransactionRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabTransactionRepo for YnabTransactionRepoImpl {
        async fn get_all(&self) -> DbResult<Vec<TransactionDetail>>;
        async fn update_all(&self, transactions: &[TransactionDetail]) -> DbResult<()>;
        async fn get_all_with_payee_id(&self, payee_id: Uuid) -> DbResult<Vec<TransactionDetail>>;
        async fn get_all_with_category_id(&self, category_id: Uuid) -> DbResult<Vec<TransactionDetail>>;
    }
}

#[async_trait]
pub trait YnabCategoryMetaRepo: DynClone + Send + Sync {
    async fn get_delta(&mut self) -> DbResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DbResult<()>;
    async fn del_delta(&mut self) -> DbResult<i64>;
    async fn get_last_saved(&mut self) -> DbResult<String>;
    async fn set_last_saved(&mut self, last_saved: String) -> DbResult<()>;
}

clone_trait_object!(YnabCategoryMetaRepo);

pub type DynYnabCategoryMetaRepo = Box<dyn YnabCategoryMetaRepo>;

#[async_trait]
pub trait YnabScheduledTransactionMetaRepo: DynClone + Send + Sync {
    async fn get_delta(&mut self) -> DbResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DbResult<()>;
    async fn del_delta(&mut self) -> DbResult<i64>;
    async fn get_last_saved(&mut self) -> DbResult<String>;
    async fn set_last_saved(&mut self, last_saved: String) -> DbResult<()>;
}

clone_trait_object!(YnabScheduledTransactionMetaRepo);

pub type DynYnabScheduledTransactionMetaRepo = Box<dyn YnabScheduledTransactionMetaRepo>;

#[async_trait]
pub trait YnabAccountMetaRepo: DynClone + Send + Sync {
    async fn get_delta(&mut self) -> DbResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DbResult<()>;
}

clone_trait_object!(YnabAccountMetaRepo);

pub type DynYnabAccountMetaRepo = Box<dyn YnabAccountMetaRepo>;

#[async_trait]
pub trait YnabPayeeMetaRepo: DynClone + Send + Sync {
    async fn get_delta(&mut self) -> DbResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DbResult<()>;
}

clone_trait_object!(YnabPayeeMetaRepo);

pub type DynYnabPayeeMetaRepo = Box<dyn YnabPayeeMetaRepo>;

#[async_trait]
pub trait YnabTransactionMetaRepo: DynClone + Send + Sync {
    async fn get_delta(&mut self) -> DbResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DbResult<()>;
}

clone_trait_object!(YnabTransactionMetaRepo);

pub type DynYnabTransactionMetaRepo = Box<dyn YnabTransactionMetaRepo>;

#[cfg(any(feature = "testutils", test))]
mod mocks {
    use super::*;
    use fake::{Fake, Faker};

    #[derive(Clone)]
    pub struct MockYnabCategoryMetaRepo {}

    impl MockYnabCategoryMetaRepo {
        pub fn new_boxed() -> Box<dyn YnabCategoryMetaRepo> {
            Box::new(Self {})
        }
    }

    #[async_trait]
    impl YnabCategoryMetaRepo for MockYnabCategoryMetaRepo {
        async fn get_delta(&mut self) -> DbResult<i64> {
            Ok(Faker.fake())
        }

        async fn set_delta(&mut self, _server_knowledge: i64) -> DbResult<()> {
            Ok(())
        }

        async fn del_delta(&mut self) -> DbResult<i64> {
            Ok(Faker.fake())
        }

        async fn get_last_saved(&mut self) -> DbResult<String> {
            Ok(Faker.fake::<chrono::NaiveDate>().to_string())
        }

        async fn set_last_saved(&mut self, _last_saved: String) -> DbResult<()> {
            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct MockYnabScheduledTransactionMetaRepo {}

    impl MockYnabScheduledTransactionMetaRepo {
        pub fn new_boxed() -> Box<dyn YnabScheduledTransactionMetaRepo> {
            Box::new(Self {})
        }
    }

    #[async_trait]
    impl YnabScheduledTransactionMetaRepo for MockYnabScheduledTransactionMetaRepo {
        async fn get_delta(&mut self) -> DbResult<i64> {
            Ok(Faker.fake())
        }

        async fn set_delta(&mut self, _server_knowledge: i64) -> DbResult<()> {
            Ok(())
        }

        async fn del_delta(&mut self) -> DbResult<i64> {
            Ok(Faker.fake())
        }

        async fn get_last_saved(&mut self) -> DbResult<String> {
            Ok(Faker.fake::<chrono::NaiveDate>().to_string())
        }

        async fn set_last_saved(&mut self, _last_saved: String) -> DbResult<()> {
            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct MockYnabAccountMetaRepo {}

    impl MockYnabAccountMetaRepo {
        pub fn new_boxed() -> Box<dyn YnabAccountMetaRepo> {
            Box::new(Self {})
        }
    }

    #[async_trait]
    impl YnabAccountMetaRepo for MockYnabAccountMetaRepo {
        async fn get_delta(&mut self) -> DbResult<i64> {
            Ok(Faker.fake())
        }

        async fn set_delta(&mut self, _server_knowledge: i64) -> DbResult<()> {
            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct MockYnabPayeeMetaRepo {}

    impl MockYnabPayeeMetaRepo {
        pub fn new_boxed() -> Box<dyn YnabPayeeMetaRepo> {
            Box::new(Self {})
        }
    }

    #[async_trait]
    impl YnabPayeeMetaRepo for MockYnabPayeeMetaRepo {
        async fn get_delta(&mut self) -> DbResult<i64> {
            Ok(Faker.fake())
        }

        async fn set_delta(&mut self, _server_knowledge: i64) -> DbResult<()> {
            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct MockYnabTransactionMetaRepo {}

    impl MockYnabTransactionMetaRepo {
        pub fn new_boxed() -> Box<dyn YnabTransactionMetaRepo> {
            Box::new(Self {})
        }
    }

    #[async_trait]
    impl YnabTransactionMetaRepo for MockYnabTransactionMetaRepo {
        async fn get_delta(&mut self) -> DbResult<i64> {
            Ok(Faker.fake())
        }

        async fn set_delta(&mut self, _server_knowledge: i64) -> DbResult<()> {
            Ok(())
        }
    }
}

#[cfg(any(feature = "testutils", test))]
pub use mocks::*;
