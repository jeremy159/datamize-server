use async_trait::async_trait;
use uuid::Uuid;
use ynab::types::Account;
use ynab::types::Category;
use ynab::types::Payee;
use ynab::types::ScheduledTransactionDetail;

use crate::error::DatamizeResult;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabCategoryRepo {
    async fn get_all(&self) -> DatamizeResult<Vec<Category>>;
    async fn get(&self, category_id: Uuid) -> DatamizeResult<Category>;
    async fn update_all(&self, categories: &[Category]) -> DatamizeResult<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabScheduledTransactionRepo {
    async fn get_all(&self) -> DatamizeResult<Vec<ScheduledTransactionDetail>>;
    async fn update_all(
        &self,
        scheduled_transactions: &[ScheduledTransactionDetail],
    ) -> DatamizeResult<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabAccountRepo {
    async fn get_all(&self) -> DatamizeResult<Vec<Account>>;
    async fn update_all(&self, accounts: &[Account]) -> DatamizeResult<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabPayeeRepo {
    async fn get_all(&self) -> DatamizeResult<Vec<Payee>>;
    async fn update_all(&self, payees: &[Payee]) -> DatamizeResult<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabCategoryMetaRepo {
    async fn get_delta(&mut self) -> DatamizeResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
    async fn del_delta(&mut self) -> DatamizeResult<i64>;
    async fn get_last_saved(&mut self) -> DatamizeResult<String>;
    async fn set_last_saved(&mut self, last_saved: String) -> DatamizeResult<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabScheduledTransactionMetaRepo {
    async fn get_delta(&mut self) -> DatamizeResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
    async fn del_delta(&mut self) -> DatamizeResult<i64>;
    async fn get_last_saved(&mut self) -> DatamizeResult<String>;
    async fn set_last_saved(&mut self, last_saved: String) -> DatamizeResult<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabAccountMetaRepo {
    async fn get_delta(&mut self) -> DatamizeResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabPayeeMetaRepo {
    async fn get_delta(&mut self) -> DatamizeResult<i64>;
    async fn set_delta(&mut self, server_knowledge: i64) -> DatamizeResult<()>;
}
