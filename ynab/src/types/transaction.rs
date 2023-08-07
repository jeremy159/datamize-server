use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Transactions/getTransactionById
pub struct TransactionSummary {
    pub id: Uuid,
    pub date: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub matched_transaction_id: Option<Uuid>,
    pub import_id: Option<Uuid>,
    pub deleted: bool,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Transactions/getTransactionById
pub struct TransactionSummary {
    pub id: Uuid,
    pub date: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub matched_transaction_id: Option<Uuid>,
    pub import_id: Option<Uuid>,
    pub deleted: bool,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseTransactionDetail {
    pub id: Uuid,
    pub date: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub matched_transaction_id: Option<Uuid>,
    pub import_id: Option<Uuid>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: String,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BaseTransactionDetail {
    pub id: Uuid,
    pub date: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub matched_transaction_id: Option<Uuid>,
    pub import_id: Option<Uuid>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: String,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum ClearedType {
    Cleared,
    Uncleared,
    Reconciled,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
#[sqlx(type_name = "cleared")]
#[sqlx(rename_all = "camelCase")]
pub enum ClearedType {
    Cleared,
    Uncleared,
    Reconciled,
}

impl fmt::Display for ClearedType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClearedType::Cleared => write!(f, "cleared"),
            ClearedType::Uncleared => write!(f, "uncleared"),
            ClearedType::Reconciled => write!(f, "reconciled"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseClearedTypeError;

impl FromStr for ClearedType {
    type Err = ParseClearedTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cleared" => Ok(ClearedType::Cleared),
            "uncleared" => Ok(ClearedType::Uncleared),
            "reconciled" => Ok(ClearedType::Reconciled),
            _ => Err(ParseClearedTypeError),
        }
    }
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct HybridTransaction {
    #[serde(flatten)]
    pub base: BaseTransactionDetail,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct HybridTransaction {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub base: BaseTransactionDetail,
}

impl AsRef<BaseTransactionDetail> for HybridTransaction {
    fn as_ref(&self) -> &BaseTransactionDetail {
        &self.base
    }
}

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridTransationsDelta {
    pub transactions: Vec<HybridTransaction>,
    pub server_knowledge: i64,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Transactions/getTransactionById
pub struct TransactionDetail {
    #[serde(flatten)]
    pub base: BaseTransactionDetail,
    pub subtransactions: Vec<SubTransaction>,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Transactions/getTransactionById
pub struct TransactionDetail {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub base: BaseTransactionDetail,
    pub subtransactions: Vec<SubTransaction>,
}

impl AsRef<BaseTransactionDetail> for TransactionDetail {
    fn as_ref(&self) -> &BaseTransactionDetail {
        &self.base
    }
}

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransationsDetailDelta {
    pub transactions: Vec<TransactionDetail>,
    pub server_knowledge: i64,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Transactions/createTransaction
pub struct SaveTransaction {
    pub account_id: Uuid,
    pub date: String,
    pub amount: i64,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub memo: Option<String>,
    pub cleared: Option<ClearedType>,
    pub approved: Option<bool>,
    pub flag_color: Option<String>,
    pub import_id: Option<Uuid>,
    pub subtransactions: Option<Vec<SaveSubTransaction>>,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Transactions/updateTransactions
pub struct UpdateTransaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub date: String,
    pub amount: i64,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub memo: Option<String>,
    pub cleared: Option<ClearedType>,
    pub approved: Option<bool>,
    pub flag_color: Option<String>,
    pub import_id: Option<Uuid>,
    pub subtransactions: Option<Vec<SaveSubTransaction>>,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubTransaction {
    pub id: Uuid,
    pub scheduled_transaction_id: Option<Uuid>,
    pub amount: i64,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Transactions/createTransaction
pub struct SaveSubTransaction {
    pub amount: i64,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub memo: Option<String>,
}
