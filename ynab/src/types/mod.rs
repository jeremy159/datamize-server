mod account;
mod budget;
mod category;
mod month;
mod payee;
mod scheduled_subtransaction;
mod transaction;

pub use account::*;
pub use budget::*;
pub use category::*;
pub use month::*;
pub use payee::*;
pub use scheduled_subtransaction::*;
pub use transaction::*;

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/User/getUser
pub struct User {
    pub id: Uuid,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Clone, Copy, Serialize)]
pub enum TransactionType {
    Unapproved,
    Uncategorized,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransactionType::Unapproved => write!(f, "unapproved"),
            TransactionType::Uncategorized => write!(f, "uncategorized"),
        }
    }
}

/// Used for transactions request, when we need to get transactions
/// from a specific sub-path.
/// This means for example using `TransactionsParentPath::Accounts("1234")` will
/// result in the api path `"/budgets/11111/accounts/1234/transactions"`.
pub enum TransactionsParentPath<T: AsRef<str>> {
    Accounts(T),
    Categories(T),
    Payees(T),
}

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Serialize, Default)]
pub struct TransactionsRequestQuery {
    pub last_knowledge_of_server: Option<i64>,
    pub since_date: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
}

impl TransactionsRequestQuery {
    pub fn with_last_knowledge(mut self, last_knowledge_of_server: Option<i64>) -> Self {
        self.last_knowledge_of_server = last_knowledge_of_server;
        self
    }

    pub fn with_date(mut self, since_date: &str) -> Self {
        self.since_date = Some(since_date.to_string());
        self
    }

    pub fn with_transaction_type(mut self, transaction_type: TransactionType) -> Self {
        self.transaction_type = Some(transaction_type);
        self
    }
}
