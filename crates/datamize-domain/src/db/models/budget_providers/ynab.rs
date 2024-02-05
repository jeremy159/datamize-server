use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;
use ynab::types::Account;
use ynab::types::Category;
use ynab::types::Payee;
use ynab::types::ScheduledTransactionDetail;
use ynab::TransactionDetail;

use crate::db::error::DbResult;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait YnabCategoryRepo: Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<Category>>;
    async fn get(&self, category_id: Uuid) -> DbResult<Category>;
    async fn update_all(&self, categories: &[Category]) -> DbResult<()>;
}

pub type DynYnabCategoryRepo = Arc<dyn YnabCategoryRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait YnabScheduledTransactionRepo: Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<ScheduledTransactionDetail>>;
    async fn update_all(
        &self,
        scheduled_transactions: &[ScheduledTransactionDetail],
    ) -> DbResult<()>;
}

pub type DynYnabScheduledTransactionRepo = Arc<dyn YnabScheduledTransactionRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait YnabAccountRepo: Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<Account>>;
    async fn update_all(&self, accounts: &[Account]) -> DbResult<()>;
}

pub type DynYnabAccountRepo = Arc<dyn YnabAccountRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait YnabPayeeRepo: Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<Payee>>;
    async fn update_all(&self, payees: &[Payee]) -> DbResult<()>;
}

pub type DynYnabPayeeRepo = Arc<dyn YnabPayeeRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait YnabTransactionRepo: Send + Sync {
    async fn get_all(&self) -> DbResult<Vec<TransactionDetail>>;
    async fn update_all(&self, transactions: &[TransactionDetail]) -> DbResult<()>;
    async fn get_all_with_payee_id(&self, payee_id: Uuid) -> DbResult<Vec<TransactionDetail>>;
    async fn get_all_with_category_id(&self, category_id: Uuid)
        -> DbResult<Vec<TransactionDetail>>;
}

pub type DynYnabTransactionRepo = Arc<dyn YnabTransactionRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait YnabCategoryMetaRepo: Send + Sync {
    async fn get_delta(&self) -> DbResult<i64>;
    async fn set_delta(&self, server_knowledge: i64) -> DbResult<()>;
    async fn del_delta(&self) -> DbResult<()>;
    async fn get_last_saved(&self) -> DbResult<String>;
    async fn set_last_saved(&self, last_saved: String) -> DbResult<()>;
}

pub type DynYnabCategoryMetaRepo = Arc<dyn YnabCategoryMetaRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait YnabScheduledTransactionMetaRepo: Send + Sync {
    async fn get_delta(&self) -> DbResult<i64>;
    async fn set_delta(&self, server_knowledge: i64) -> DbResult<()>;
    async fn del_delta(&self) -> DbResult<()>;
    async fn get_last_saved(&self) -> DbResult<String>;
    async fn set_last_saved(&self, last_saved: String) -> DbResult<()>;
}

pub type DynYnabScheduledTransactionMetaRepo = Arc<dyn YnabScheduledTransactionMetaRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait YnabAccountMetaRepo: Send + Sync {
    async fn get_delta(&self) -> DbResult<i64>;
    async fn set_delta(&self, server_knowledge: i64) -> DbResult<()>;
}

pub type DynYnabAccountMetaRepo = Arc<dyn YnabAccountMetaRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait YnabPayeeMetaRepo: Send + Sync {
    async fn get_delta(&self) -> DbResult<i64>;
    async fn set_delta(&self, server_knowledge: i64) -> DbResult<()>;
}

pub type DynYnabPayeeMetaRepo = Arc<dyn YnabPayeeMetaRepo>;

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait YnabTransactionMetaRepo: Send + Sync {
    async fn get_delta(&self) -> DbResult<i64>;
    async fn set_delta(&self, server_knowledge: i64) -> DbResult<()>;
}

pub type DynYnabTransactionMetaRepo = Arc<dyn YnabTransactionMetaRepo>;
