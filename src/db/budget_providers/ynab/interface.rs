use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use uuid::Uuid;
use ynab::types::Account;
use ynab::types::Category;
use ynab::types::Payee;
use ynab::types::ScheduledTransactionDetail;

use crate::error::DatamizeResult;

#[async_trait]
pub trait YnabCategoryRepo: DynClone {
    async fn get_all(&self) -> DatamizeResult<Vec<Category>>;
    async fn get(&self, category_id: Uuid) -> DatamizeResult<Category>;
    async fn update_all(&self, categories: &[Category]) -> DatamizeResult<()>;
}

clone_trait_object!(YnabCategoryRepo);

pub type DynYnabCategoryRepo = Box<dyn YnabCategoryRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub YnabCategoryRepoImpl {}

    impl Clone for YnabCategoryRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabCategoryRepo for YnabCategoryRepoImpl {
        async fn get_all(&self) -> DatamizeResult<Vec<Category>>;
        async fn get(&self, category_id: Uuid) -> DatamizeResult<Category>;
        async fn update_all(&self, categories: &[Category]) -> DatamizeResult<()>;
    }
}

#[async_trait]
pub trait YnabScheduledTransactionRepo: DynClone {
    async fn get_all(&self) -> DatamizeResult<Vec<ScheduledTransactionDetail>>;
    async fn update_all(
        &self,
        scheduled_transactions: &[ScheduledTransactionDetail],
    ) -> DatamizeResult<()>;
}

clone_trait_object!(YnabScheduledTransactionRepo);

pub type DynYnabScheduledTransactionRepo = Box<dyn YnabScheduledTransactionRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub YnabScheduledTransactionRepoImpl {}

    impl Clone for YnabScheduledTransactionRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabScheduledTransactionRepo for YnabScheduledTransactionRepoImpl {
        async fn get_all(&self) -> DatamizeResult<Vec<ScheduledTransactionDetail>>;
        async fn update_all(&self, scheduled_transactions: &[ScheduledTransactionDetail]) -> DatamizeResult<()>;
    }
}

#[async_trait]
pub trait YnabAccountRepo: DynClone {
    async fn get_all(&self) -> DatamizeResult<Vec<Account>>;
    async fn update_all(&self, accounts: &[Account]) -> DatamizeResult<()>;
}

clone_trait_object!(YnabAccountRepo);

pub type DynYnabAccountRepo = Box<dyn YnabAccountRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub YnabAccountRepoImpl {}

    impl Clone for YnabAccountRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabAccountRepo for YnabAccountRepoImpl {
        async fn get_all(&self) -> DatamizeResult<Vec<Account>>;
        async fn update_all(&self, accounts: &[Account]) -> DatamizeResult<()>;
    }
}

#[async_trait]
pub trait YnabPayeeRepo: DynClone {
    async fn get_all(&self) -> DatamizeResult<Vec<Payee>>;
    async fn update_all(&self, payees: &[Payee]) -> DatamizeResult<()>;
}

clone_trait_object!(YnabPayeeRepo);

pub type DynYnabPayeeRepo = Box<dyn YnabPayeeRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub YnabPayeeRepoImpl {}

    impl Clone for YnabPayeeRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabPayeeRepo for YnabPayeeRepoImpl {
        async fn get_all(&self) -> DatamizeResult<Vec<Payee>>;
        async fn update_all(&self, payees: &[Payee]) -> DatamizeResult<()>;
    }
}

#[async_trait]
pub trait YnabCategoryMetaRepo: DynClone {
    async fn get_delta(&mut self) -> DatamizeResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
    async fn del_delta(&mut self) -> DatamizeResult<i64>;
    async fn get_last_saved(&mut self) -> DatamizeResult<String>;
    async fn set_last_saved(&mut self, last_saved: String) -> DatamizeResult<()>;
}

clone_trait_object!(YnabCategoryMetaRepo);

pub type DynYnabCategoryMetaRepo = Box<dyn YnabCategoryMetaRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub YnabCategoryMetaRepoImpl {}

    impl Clone for YnabCategoryMetaRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabCategoryMetaRepo for YnabCategoryMetaRepoImpl {
        async fn get_delta(&mut self) -> DatamizeResult<i64>;
        async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
        async fn del_delta(&mut self) -> DatamizeResult<i64>;
        async fn get_last_saved(&mut self) -> DatamizeResult<String>;
        async fn set_last_saved(&mut self, last_saved: String) -> DatamizeResult<()>;
    }
}

#[async_trait]
pub trait YnabScheduledTransactionMetaRepo: DynClone {
    async fn get_delta(&mut self) -> DatamizeResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
    async fn del_delta(&mut self) -> DatamizeResult<i64>;
    async fn get_last_saved(&mut self) -> DatamizeResult<String>;
    async fn set_last_saved(&mut self, last_saved: String) -> DatamizeResult<()>;
}

clone_trait_object!(YnabScheduledTransactionMetaRepo);

pub type DynYnabScheduledTransactionMetaRepo =
    Box<dyn YnabScheduledTransactionMetaRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub YnabScheduledTransactionMetaRepoImpl {}

    impl Clone for YnabScheduledTransactionMetaRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabScheduledTransactionMetaRepo for YnabScheduledTransactionMetaRepoImpl {
        async fn get_delta(&mut self) -> DatamizeResult<i64>;
        async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
        async fn del_delta(&mut self) -> DatamizeResult<i64>;
        async fn get_last_saved(&mut self) -> DatamizeResult<String>;
        async fn set_last_saved(&mut self, last_saved: String) -> DatamizeResult<()>;
    }
}

#[async_trait]
pub trait YnabAccountMetaRepo: DynClone {
    async fn get_delta(&mut self) -> DatamizeResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
}

clone_trait_object!(YnabAccountMetaRepo);

pub type DynYnabAccountMetaRepo = Box<dyn YnabAccountMetaRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub YnabAccountMetaRepoImpl {}

    impl Clone for YnabAccountMetaRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabAccountMetaRepo for YnabAccountMetaRepoImpl {
        async fn get_delta(&mut self) -> DatamizeResult<i64>;
        async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
    }
}

#[async_trait]
pub trait YnabPayeeMetaRepo: DynClone {
    async fn get_delta(&mut self) -> DatamizeResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
}

clone_trait_object!(YnabPayeeMetaRepo);

pub type DynYnabPayeeMetaRepo = Box<dyn YnabPayeeMetaRepo + Send + Sync>;

#[cfg(test)]
mockall::mock! {
    pub YnabPayeeMetaRepoImpl {}

    impl Clone for YnabPayeeMetaRepoImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabPayeeMetaRepo for YnabPayeeMetaRepoImpl {
        async fn get_delta(&mut self) -> DatamizeResult<i64>;
        async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
    }
}
